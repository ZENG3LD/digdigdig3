#![allow(clippy::all)]
//! Poseidon2 hash function over the Goldilocks field.
//!
//! Parameters (the "Goldilocks" variant used by Lighter/poseidon_crypto):
//! - Field: GF(2^64 - 2^32 + 1)
//! - Width (t): 12
//! - Rate: 8, Capacity: 4
//! - Full rounds: 4 + 4 = 8 total
//! - Partial rounds: 22
//! - S-box: x^7
//!
//! The permutation structure:
//!   external_linear_layer(state)
//!   4 full rounds
//!   22 partial rounds
//!   4 full rounds
//!
//! Primary function: `hash_to_quintic_extension(inputs) -> GFp5`
//! Used for transaction hashing and Schnorr challenge computation.

use super::goldilocks::{GFp, pow7};
use super::gfp5::GFp5;

/// External (full) round constants — 8 rounds × 12 elements.
/// From elliottech/poseidon_crypto (poseidon2_goldilocks variant).
const EXTERNAL_CONSTANTS: [[u64; 12]; 8] = [
    [
        15492826721047263190, 11728330187201910315, 8836021247773420868, 16777404051263952451,
        5510875212538051896,  6173089941271892285,  2927757366422211339, 10340958981325008808,
        8541987352684552425,  9739599543776434497, 15073950188101532019, 12084856431752384512,
    ],
    [
        4584713381960671270,  8807052963476652830,    54136601502601741,  4872702333905478703,
        5551030319979516287, 12889366755535460989, 16329242193178844328,   412018088475211848,
        10505784623379650541, 9758812378619434837,  7421979329386275117,   375240370024755551,
    ],
    [
        3331431125640721931, 15684937309956309981,   578521833432107983, 14379242000670861838,
        17922409828154900976, 8153494278429192257, 15904673920630731971, 11217863998460634216,
        3301540195510742136,  9937973023749922003,  3059102938155026419,  1895288289490976132,
    ],
    [
        5580912693628927540, 10064804080494788323,  9582481583369602410, 10186259561546797986,
        247426333829703916, 13193193905461376067,  6386232593701758044, 17954717245501896472,
        1531720443376282699,  2455761864255501970, 11234429217864304495,  4746959618548874102,
    ],
    [
        13571697342473846203, 17477857865056504753, 15963032953523553760, 16033593225279635898,
        14252634232868282405, 8219748254835277737,  7459165569491914711, 15855939513193752003,
        16788866461340278896, 7102224659693946577,  3024718005636976471, 13695468978618890430,
    ],
    [
        8214202050877825436,  2670727992739346204, 16259532062589659211, 11869922396257088411,
        3179482916972760137, 13525476046633427808,  3217337278042947412, 14494689598654046340,
        15837379330312175383, 8029037639801151344,  2153456285263517937,  8301106462311849241,
    ],
    [
        13294194396455217955, 17394768489610594315, 12847609130464867455, 14015739446356528640,
        5879251655839607853,  9747000124977436185,  8950393546890284269, 10765765936405694368,
        14695323910334139959, 16366254691123000864, 15292774414889043182, 10910394433429313384,
    ],
    [
        17253424460214596184, 3442854447664030446,  3005570425335613727, 10859158614900201063,
        9763230642109343539,  6647722546511515039,   909012944955815706, 18101204076790399111,
        11588128829349125809, 15863878496612806566,  5201119062417750399,   176665553780565743,
    ],
];

/// Internal (partial) round constants — applied only to state[0].
const INTERNAL_CONSTANTS: [u64; 22] = [
    11921381764981422944, 10318423381711320787,  8291411502347000766,   229948027109387563,
    9152521390190983261,  7129306032690285515, 15395989607365232011,  8641397269074305925,
    17256848792241043600,  6046475228902245682, 12041608676381094092, 12785542378683951657,
    14546032085337914034,  3304199118235116851, 16499627707072547655, 10386478025625759321,
    13475579315436919170, 16042710511297532028,  1411266850385657080,  9024840976168649958,
    14047056970978379368,   838728605080212101,
];

/// Internal diagonal matrix constants for the partial rounds linear layer.
/// These are in Montgomery form as used by the Go reference library.
/// We store them as canonical u64 and convert via GFp::from_u64_reduce.
const MATRIX_DIAG_12_RAW: [u64; 12] = [
    0xc3b6c08e23ba9300, 0xd84b5de94a324fb6, 0x0d0c371c5b35b84f, 0x7964f570e7188037,
    0x5daf18bbd996604b, 0x6743bc47b9595257, 0x5528b9362c59bb70, 0xac45e25b7127b68b,
    0xa2077d7dfbb606b5, 0xf3faac6faee378ae, 0x0c6388b51545e883, 0xd27dbb6944917b60,
];

/// Poseidon2 state (12 Goldilocks field elements).
struct State([GFp; 12]);

impl State {
    fn new() -> Self {
        State([GFp::ZERO; 12])
    }

    /// Add external round constants (full round).
    fn add_external_rc(&mut self, round: usize) {
        let rc = &EXTERNAL_CONSTANTS[round];
        for i in 0..12 {
            self.0[i] += GFp::from_u64_reduce(rc[i]);
        }
    }

    /// Apply S-box (x^7) to all 12 elements.
    fn sbox_full(&mut self) {
        for i in 0..12 {
            self.0[i] = pow7(self.0[i]);
        }
    }

    /// Apply S-box only to state[0].
    fn sbox_partial(&mut self) {
        self.0[0] = pow7(self.0[0]);
    }

    /// External linear layer (MDS matrix for WIDTH=12).
    ///
    /// The Poseidon2 external linear layer for WIDTH=12 processes in 3 groups of 4,
    /// then mixes across groups.
    fn external_linear_layer(&mut self) {
        // Process each group of 4 independently
        for group in 0..3 {
            let base = group * 4;
            let a = self.0[base];
            let b = self.0[base + 1];
            let c = self.0[base + 2];
            let d = self.0[base + 3];

            let t0 = a + b;
            let t1 = c + d;
            let t2 = t0 + t1;
            let t3 = t2 + b;      // t2 + b
            let t4 = t2 + d;      // t2 + d
            // a' = t3 + t0 = t2 + b + a + b = (a+b+c+d) + a + 2b - (a+b) = (a+b+c+d) + (a+b) + b
            // Simplify:
            // t3 = (a+b+c+d) + b
            // t4 = (a+b+c+d) + d
            // t5 = 2*a
            // t6 = 2*c
            // a' = t3 + t0  = (a+b+c+d) + b + (a+b) = 2a+3b+c+d
            // b' = t6 + t3  = 2c + (a+b+c+d) + b = a+2b+3c+d
            // c' = t1 + t4  = (c+d) + (a+b+c+d) + d = a+b+2c+3d  (actually check below)
            // d' = t5 + t4  = 2a + (a+b+c+d) + d = 3a+b+c+2d

            // From Poseidon2 paper / Go SDK implementation:
            // For each group [a,b,c,d]:
            //   t0 = a+b; t1 = c+d; t2 = t0+t1; t3 = t2+b; t4 = t2+d
            //   a' = t3 + t0 = t2+b+t0 = (a+b+c+d)+b+(a+b) = 2a+3b+c+d
            //   b' = t6+t3 = 2c + t2+b = 2c+(a+b+c+d)+b = a+2b+3c+d
            //   c' = t1+t4 = (c+d)+t2+d = (c+d)+(a+b+c+d)+d = a+b+2c+3d
            //     Wait: t4 = t2+d = (a+b+c+d)+d = a+b+c+2d
            //   c' = t1+t4 = (c+d)+(a+b+c+2d) = a+b+2c+3d  ✓
            //   d' = t5+t4 = 2a+(a+b+c+2d) = 3a+b+c+2d  ✓
            let t5 = a.double();
            let t6 = c.double();
            self.0[base]     = t3 + t0;   // 2a+3b+c+d
            self.0[base + 1] = t6 + t3;   // a+2b+3c+d... wait t6=2c so this is 2c+a+2b+c+d = a+2b+3c+d ✓
            self.0[base + 2] = t1 + t4;   // a+b+2c+3d ✓
            self.0[base + 3] = t5 + t4;   // 3a+b+c+2d ✓
        }

        // Cross-group mixing:
        //   finalsum[i] = s'[i] + t'[i] + u'[i]
        //   result[i]   = s'[i] + finalsum[i]  = 2*s'[i] + t'[i] + u'[i]
        //   result[4+i] = t'[i] + finalsum[i]  = s'[i] + 2*t'[i] + u'[i]
        //   result[8+i] = u'[i] + finalsum[i]  = s'[i] + t'[i] + 2*u'[i]
        let s2 = [self.0[0], self.0[1], self.0[2], self.0[3]];
        let t2 = [self.0[4], self.0[5], self.0[6], self.0[7]];
        let u2 = [self.0[8], self.0[9], self.0[10], self.0[11]];
        for i in 0..4 {
            let total = s2[i] + t2[i] + u2[i];
            self.0[i]     = s2[i] + total;
            self.0[4 + i] = t2[i] + total;
            self.0[8 + i] = u2[i] + total;
        }
    }

    /// Internal linear layer (partial rounds).
    /// state[i] = state[i] * diag[i] + sum(state)
    fn internal_linear_layer(&mut self) {
        // Compute sum of all elements
        let mut sum = GFp::ZERO;
        for i in 0..12 {
            sum += self.0[i];
        }
        // Multiply each element by its diagonal constant, then add the sum
        for i in 0..12 {
            // The diagonal values from Go are in Montgomery form already.
            // We treat them as raw Montgomery values — but our GFp stores in Montgomery
            // form too, so we need to be consistent.
            // The matrix diagonal values from the Go SDK are the actual Montgomery
            // representations, so we store them directly in GFp's internal format.
            let d = GFp(MATRIX_DIAG_12_RAW[i]);
            self.0[i] = self.0[i] * d + sum;
        }
    }

    /// Full round: add round constants, S-box full, external linear layer.
    fn full_round(&mut self, round: usize) {
        self.add_external_rc(round);
        self.sbox_full();
        self.external_linear_layer();
    }

    /// Partial round: add constant to state[0], S-box on state[0], internal linear layer.
    fn partial_round(&mut self, round: usize) {
        self.0[0] += GFp::from_u64_reduce(INTERNAL_CONSTANTS[round]);
        self.sbox_partial();
        self.internal_linear_layer();
    }

    /// Full permutation: 4 full + 22 partial + 4 full rounds.
    fn permute(&mut self) {
        // Initial external linear layer (before first full rounds)
        self.external_linear_layer();

        // 4 full rounds
        for r in 0..4 {
            self.full_round(r);
        }

        // 22 partial rounds
        for r in 0..22 {
            self.partial_round(r);
        }

        // 4 full rounds
        for r in 4..8 {
            self.full_round(r);
        }
    }
}

/// Poseidon2 hash: absorb input field elements and return state[0..4] (capacity).
/// Uses sponge construction: absorb RATE=8 elements at a time.
///
/// IMPORTANT: The sponge overwrites state[0..chunk_len] with the chunk contents,
/// then calls permute. It does NOT XOR into existing state.
pub fn poseidon2_hash(inputs: &[GFp]) -> [GFp; 4] {
    let mut state = State::new();
    let rate = 8;

    // Absorb in chunks of RATE=8
    let mut i = 0;
    while i < inputs.len() {
        let chunk_len = (inputs.len() - i).min(rate);
        for j in 0..chunk_len {
            state.0[j] = inputs[i + j];
        }
        state.permute();
        i += chunk_len;
    }

    // If inputs is empty, still do one permutation
    if inputs.is_empty() {
        state.permute();
    }

    // Return capacity portion (state[0..4])
    [state.0[0], state.0[1], state.0[2], state.0[3]]
}

/// Hash input field elements to a GFp5 quintic extension element.
/// This is the primary function used for transaction signing.
///
/// Absorbs inputs in chunks of RATE=8, returns state[0..5] as GFp5.
/// Used for: tx hash, auth token hash, Schnorr challenge computation.
pub fn hash_to_quintic_extension(inputs: &[GFp]) -> GFp5 {
    let mut state = State::new();
    let rate = 8;

    let mut i = 0;
    while i < inputs.len() {
        let chunk_len = (inputs.len() - i).min(rate);
        for j in 0..chunk_len {
            state.0[j] = inputs[i + j];
        }
        state.permute();
        i += chunk_len;
    }

    if inputs.is_empty() {
        state.permute();
    }

    GFp5([state.0[0], state.0[1], state.0[2], state.0[3], state.0[4]])
}

/// Convert bytes to GoldilocksField elements (little-endian 8-byte chunks).
/// Used for auth token hashing: the ASCII string is split into 8-byte chunks.
/// Each chunk is interpreted as a little-endian u64, must be < p.
/// For strings where all bytes < 0x80, all chunks will be < p.
pub fn bytes_to_field_elements(data: &[u8]) -> Vec<GFp> {
    let mut result = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let mut buf = [0u8; 8];
        let chunk_len = (data.len() - i).min(8);
        buf[..chunk_len].copy_from_slice(&data[i..i + chunk_len]);
        let v = u64::from_le_bytes(buf);
        // Trust ASCII strings are < p (they are: ASCII is 7-bit, max 8-byte chunk is
        // 0x7F7F7F7F7F7F7F7F which is well below p = 0xFFFFFFFF00000001)
        result.push(GFp::from_canonical_u64(v));
        i += 8;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test vector from the Go poseidon_crypto library (TestPermute).
    /// Input 12 elements, expected output 12 elements.
    #[test]
    fn test_permute_vector() {
        let input: [u64; 12] = [
            5417613058500526590, 2481548824842427254, 6473243198879784792, 1720313757066167274,
            2806320291675974571, 7407976414706455446, 1105257841424046885, 7613435757403328049,
            3376066686066811538, 5888575799323675710, 6689309723188675948, 2468250420241012720,
        ];
        let expected: [u64; 12] = [
            5364184781011389007, 15309475861242939136, 5983386513087443499,  886942118604446276,
            14903657885227062600,  7742650891575941298,  1962182278500985790, 10213480816595178755,
            3510799061817443836,  4610029967627506430,  7566382334276534836,  2288460879362380348,
        ];

        let mut state = State::new();
        for i in 0..12 {
            state.0[i] = GFp::from_canonical_u64(input[i]);
        }
        state.permute();

        for i in 0..12 {
            assert_eq!(
                state.0[i].to_u64(),
                expected[i],
                "permute output[{}] mismatch: got {}, expected {}",
                i,
                state.0[i].to_u64(),
                expected[i]
            );
        }
    }

    /// Test vector from Go TestHashToQuinticExtension.
    #[test]
    fn test_hash_to_quintic_extension_vector() {
        let input_u64: [u64; 7] = [
            3451004116618606032, 11263134342958518251, 10957204882857370932, 5369763041201481933,
            7695734348563036858,  1393419330378128434,  7387917082382606332,
        ];
        let expected: [u64; 5] = [
            17992684813643984528, 5243896189906434327, 7705560276311184368, 2785244775876017560, 14449776097783372302,
        ];

        let inputs: Vec<GFp> = input_u64.iter().map(|&v| GFp::from_canonical_u64(v)).collect();
        let result = hash_to_quintic_extension(&inputs);

        for i in 0..5 {
            assert_eq!(
                result.0[i].to_u64(),
                expected[i],
                "hash_to_quintic_extension[{}] mismatch: got {}, expected {}",
                i,
                result.0[i].to_u64(),
                expected[i]
            );
        }
    }

    #[test]
    fn test_bytes_to_field_elements() {
        let data = b"hello";
        let elems = bytes_to_field_elements(data);
        assert_eq!(elems.len(), 1, "5 bytes = 1 field element");
        // "hello" as little-endian u64 = 0x6F6C6C6568 = 476399060072
        let expected = u64::from_le_bytes([b'h', b'e', b'l', b'l', b'o', 0, 0, 0]);
        assert_eq!(elems[0].to_u64(), expected);
    }
}
