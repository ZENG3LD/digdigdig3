# FAA NASSTATUS API - Coverage

## Geographic Coverage

### Primary Coverage
**United States (Continental + Alaska + Hawaii)**

The FAA NASSTATUS API covers airports in:
- **Continental United States**: All major commercial airports
- **Alaska**: Major hubs (ANC, FAI)
- **Hawaii**: Major islands (HNL, OGG, KOA, LIH, ITO)
- **US Territories**: Limited coverage (see below)

---

## Airport Tiers

### Large Hub Airports (Full Coverage)
Airports with >1% of US passenger traffic. Full delay reporting, high priority.

**Examples**:
- **ATL** - Hartsfield-Jackson Atlanta International
- **ORD** - Chicago O'Hare International
- **LAX** - Los Angeles International
- **DFW** - Dallas/Fort Worth International
- **DEN** - Denver International
- **JFK** - John F. Kennedy International (New York)
- **SFO** - San Francisco International
- **LAS** - Harry Reid International (Las Vegas)
- **SEA** - Seattle-Tacoma International
- **MCO** - Orlando International
- **EWR** - Newark Liberty International
- **CLT** - Charlotte Douglas International
- **PHX** - Phoenix Sky Harbor International
- **IAH** - George Bush Intercontinental (Houston)
- **MIA** - Miami International
- **BOS** - Boston Logan International
- **FLL** - Fort Lauderdale-Hollywood International
- **MSP** - Minneapolis-St. Paul International
- **DTW** - Detroit Metropolitan Wayne County
- **PHL** - Philadelphia International
- **LGA** - LaGuardia (New York)
- **BWI** - Baltimore/Washington International
- **SLC** - Salt Lake City International
- **SAN** - San Diego International
- **DCA** - Ronald Reagan Washington National
- **TPA** - Tampa International
- **PDX** - Portland International
- **STL** - St. Louis Lambert International
- **HNL** - Daniel K. Inouye International (Honolulu)

**Total**: ~30 airports

---

### Medium Hub Airports (Good Coverage)
Airports with 0.25-1% of US passenger traffic. Delay reporting during significant events.

**Examples**:
- **AUS** - Austin-Bergstrom International
- **BNA** - Nashville International
- **OAK** - Oakland International
- **RDU** - Raleigh-Durham International
- **SJC** - San Jose International
- **SMF** - Sacramento International
- **SNA** - John Wayne Airport (Orange County)
- **MCI** - Kansas City International
- **CLE** - Cleveland Hopkins International
- **PIT** - Pittsburgh International
- **IND** - Indianapolis International
- **CMH** - John Glenn Columbus International
- **CVG** - Cincinnati/Northern Kentucky International
- **MKE** - Milwaukee Mitchell International
- **PBI** - Palm Beach International
- **RSW** - Southwest Florida International
- **SAT** - San Antonio International
- **BDL** - Bradley International (Hartford)
- **BUF** - Buffalo Niagara International
- **OMA** - Eppley Airfield (Omaha)
- **ANC** - Ted Stevens Anchorage International
- **ABQ** - Albuquerque International Sunport
- **BUR** - Hollywood Burbank Airport
- **ONT** - Ontario International
- **SJU** - Luis Muñoz Marín International (Puerto Rico)

**Total**: ~50 airports

---

### Small Hub Airports (Moderate Coverage)
Airports with 0.05-0.25% of US passenger traffic. Coverage during major delays or closures.

**Examples**:
- **RNO** - Reno-Tahoe International
- **TUS** - Tucson International
- **ELP** - El Paso International
- **OKC** - Will Rogers World Airport
- **TUL** - Tulsa International
- **PVD** - Rhode Island T.F. Green International
- **MHT** - Manchester-Boston Regional
- **ALB** - Albany International
- **SYR** - Syracuse Hancock International
- **ROC** - Greater Rochester International
- **RIC** - Richmond International
- **GSO** - Piedmont Triad International
- **SAV** - Savannah/Hilton Head International
- **DAY** - Dayton International
- **JAX** - Jacksonville International
- **GRR** - Gerald R. Ford International (Grand Rapids)
- **DSM** - Des Moines International
- **MAF** - Midland International Air and Space Port
- **BOI** - Boise Airport
- **FAT** - Fresno Yosemite International
- **ICT** - Wichita Dwight D. Eisenhower National

**Total**: ~75 airports

---

### Non-Hub & Regional Airports (Limited Coverage)
Small commercial airports. Rarely appear in NASSTATUS unless:
- Complete closure
- Severe weather event affecting region
- Emergency situation

**Examples**:
- **MMU** - Morristown Municipal (closure example in API response)
- **STX** - Cyril E. King Airport (US Virgin Islands, closure example)
- Hundreds of smaller airports with minimal coverage

**Total**: Thousands of airports, but most not tracked in NASSTATUS

---

## US Territories

### Puerto Rico
- **SJU** - Luis Muñoz Marín International (San Juan) - **Full coverage**
- **BQN** - Rafael Hernández Airport (Aguadilla) - Limited
- **PSE** - Mercedita Airport (Ponce) - Limited

### US Virgin Islands
- **STT** - Cyril E. King Airport (St. Thomas) - Moderate
- **STX** - Henry E. Rohlsen Airport (St. Croix) - Moderate (closure example in API)

### Guam
- **GUM** - Antonio B. Won Pat International - Moderate coverage

### American Samoa
- **PPG** - Pago Pago International - Limited coverage

### Northern Mariana Islands
- **SPN** - Saipan International - Limited coverage

---

## Military Airports

### Civilian-Military Joint Use
Some military airports with civilian operations may appear:
- **HNL** (Joint use with Hickam AFB)
- Limited reporting on purely military bases

**Most military airports are NOT covered** by NASSTATUS.

---

## General Aviation Airports

### Coverage
**Very limited to none.**

The NASSTATUS API focuses on:
- Commercial passenger service
- Large cargo operations
- High-volume traffic

**General aviation airports** (private planes, flight schools) are not typically included unless:
- Major event (complete closure due to emergency)
- Located in airspace affected by AFP

---

## International Airports

### Coverage
**None.**

The FAA NASSTATUS API covers only:
- US domestic airports
- US territories

**Not covered**:
- Canadian airports
- Mexican airports
- Caribbean (non-US territory)
- Any other international locations

**For international airport status**, use:
- ICAO sources
- Individual country aviation authorities
- FlightAware, FlightRadar24 (commercial services)

---

## Data Quality by Airport Type

| Airport Type | Delay Reporting | Closure Reporting | Weather Data | Update Frequency |
|--------------|-----------------|-------------------|--------------|------------------|
| Large Hub | Excellent | Excellent | Good (legacy) | Real-time (<2 min) |
| Medium Hub | Good | Excellent | Moderate | Real-time (<5 min) |
| Small Hub | Moderate | Good | Limited | Intermittent |
| Non-Hub | Rare | Moderate | None | Intermittent |
| General Aviation | Very Rare | Rare | None | Rare |
| Military | Limited | Limited | None | Varies |

---

## Regional Bias

### Highest Coverage Density
- **Northeast Corridor**: NYC, BOS, PHL, DCA, BWI (excellent coverage)
- **California**: LAX, SFO, SAN, SJC, OAK, ONT, BUR (excellent)
- **Texas**: DFW, IAH, AUS, SAT, HOU (excellent)
- **Florida**: MIA, MCO, TPA, FLL, PBI, RSW (excellent)
- **Chicago**: ORD, MDW (excellent)

### Moderate Coverage
- **Mountain West**: DEN, SLC, PHX, LAS, ANC
- **Pacific Northwest**: SEA, PDX
- **Midwest**: MSP, DTW, CLE, STL, MCI, IND
- **Southeast**: ATL, CLT, BNA, RDU, JAX, SAV

### Lower Coverage Density
- **Great Plains**: Fewer large airports, sparse coverage
- **Mountain States**: Low population density, fewer commercial airports
- **Alaska (outside ANC/FAI)**: Limited commercial service
- **Rural areas**: Minimal coverage

---

## Coverage Scope Summary

| Category | Estimated Count | Percentage Covered |
|----------|-----------------|-------------------|
| Large Hub | ~30 | 100% |
| Medium Hub | ~50 | 95%+ |
| Small Hub | ~75 | 70-80% |
| Non-Hub Commercial | ~200 | 20-30% |
| General Aviation | ~5,000+ | <1% |
| Military | ~500 | <5% |
| **Total US Airports** | **~19,000** | **<5% overall** |

**However**, the covered airports handle **>95% of US passenger traffic** and **>90% of commercial operations**.

---

## Airport List (Top 50 by Coverage)

Based on expected NASSTATUS coverage (not exhaustive):

1. ATL - Atlanta
2. DFW - Dallas/Fort Worth
3. DEN - Denver
4. ORD - Chicago O'Hare
5. LAX - Los Angeles
6. CLT - Charlotte
7. MCO - Orlando
8. LAS - Las Vegas
9. PHX - Phoenix
10. MIA - Miami
11. SEA - Seattle
12. EWR - Newark
13. SFO - San Francisco
14. IAH - Houston
15. BOS - Boston
16. FLL - Fort Lauderdale
17. MSP - Minneapolis
18. DTW - Detroit
19. PHL - Philadelphia
20. LGA - New York LaGuardia
21. BWI - Baltimore
22. SLC - Salt Lake City
23. SAN - San Diego
24. DCA - Washington National
25. MDW - Chicago Midway
26. TPA - Tampa
27. PDX - Portland
28. STL - St. Louis
29. HNL - Honolulu
30. JFK - New York JFK
31. BNA - Nashville
32. AUS - Austin
33. OAK - Oakland
34. RDU - Raleigh-Durham
35. SJC - San Jose
36. SMF - Sacramento
37. SNA - Orange County
38. MCI - Kansas City
39. CLE - Cleveland
40. PIT - Pittsburgh
41. IND - Indianapolis
42. CMH - Columbus
43. CVG - Cincinnati
44. MKE - Milwaukee
45. PBI - Palm Beach
46. RSW - Fort Myers
47. SAT - San Antonio
48. BDL - Hartford
49. BUF - Buffalo
50. OMA - Omaha

**This list represents high-probability coverage**, not guaranteed inclusion in every NASSTATUS response.

---

## Coverage Limitations

### What NASSTATUS Does NOT Cover

1. **International airports** (outside US/territories)
2. **Small general aviation airports** (private, recreational)
3. **Historical data** (only current status)
4. **Future predictions** (no forecasting)
5. **Flight-specific delays** (only airport-wide status)
6. **Individual airline operations** (no carrier-specific data)
7. **Detailed weather forecasts** (limited weather data)
8. **Military operations** (security restrictions)
9. **Cargo-only airports** (unless also handle passengers)
10. **Heliports and seaplane bases** (not tracked)

---

## Alternative Data Sources

### For Broader Coverage

| Requirement | Alternative API | Coverage |
|-------------|-----------------|----------|
| International airports | FlightAware, FlightRadar24 | Worldwide |
| General aviation | ForeFlight, DUATS | US GA airports |
| Weather data | Aviation Weather Center (aviationweather.gov) | US + worldwide |
| Flight-specific delays | FlightStats, FlightAware | Worldwide flights |
| Historical data | Bureau of Transportation Statistics | US commercial flights |
| Military | None (restricted) | N/A |

---

## Data Freshness by Region

### Update Frequency
- **Major hubs (top 30)**: Real-time (<2 minutes)
- **Medium hubs**: Near real-time (<5 minutes)
- **Small hubs**: Intermittent (5-15 minutes when active)
- **Regional airports**: Infrequent (only major events)

### Time Zone Considerations
- **All timestamps in GMT/UTC** (no timezone conversion needed)
- Covers all US time zones: EST, CST, MST, PST, AKST, HST
- Daylight saving time changes do not affect API (UTC-based)

---

## Summary

| Feature | Details |
|---------|---------|
| Primary coverage | US domestic + territories |
| Total airports tracked | ~200 (commercial) |
| Large hub coverage | 100% (~30 airports) |
| Medium hub coverage | 95% (~50 airports) |
| Passenger traffic coverage | >95% of US passengers |
| International coverage | None |
| General aviation | <1% coverage |
| Data freshness | Real-time for major hubs |
| Historical data | Not available |
| Regional bias | Higher coverage in dense urban areas |

**NASSTATUS is ideal for monitoring major US commercial airports** but insufficient for:
- Small regional airports
- International operations
- General aviation
- Historical analysis
