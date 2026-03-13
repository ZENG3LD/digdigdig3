# Wave 4 Endpoint Gap Analysis — Batches 28–30
## Sanctions/Legal · Corporate/Trade · Demographics/Governance/News

**Date:** 2026-03-13
**Base path:** `digdigdig3/src/`
**Scope:** 16 connectors across 3 batches

---

## Batch 28 — Sanctions/Legal

---

### 1. INTERPOL (`intelligence_feeds/sanctions/interpol/endpoints.rs`)

**Base URL implemented:** `https://ws-public.interpol.int/notices/v1`

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| Red Notices | `GET /red` — search red notices | YES | |
| Red Notices | `GET /red/{notice_id}` — get red notice detail | YES | |
| Red Notices | `GET /red/{notice_id}/images` — get red notice images | YES | |
| Yellow Notices | `GET /yellow` — search yellow notices | YES | |
| Yellow Notices | `GET /yellow/{notice_id}` — get yellow notice detail | NO | Missing individual detail endpoint |
| Yellow Notices | `GET /yellow/{notice_id}/images` — get yellow notice images | NO | Missing images endpoint |
| UN Notices | `GET /un` — search UN Security Council notices | YES | Conflated as single enum variant |
| UN Notices | `GET /un/persons` — UN persons specifically | NO | API distinguishes persons vs entities |
| UN Notices | `GET /un/entities` — UN entities specifically | NO | API distinguishes persons vs entities |
| UN Notices | `GET /un/persons/{notice_id}` — UN person detail | NO | Missing |
| UN Notices | `GET /un/entities/{notice_id}` — UN entity detail | NO | Missing |
| Green Notices | `GET /green` — persons with criminal records/threats | NO | Not a public API endpoint (police-only) |
| Blue Notices | `GET /blue` — collect info on persons | NO | Not a public API endpoint (police-only) |
| Orange Notices | `GET /orange` — serious imminent threats | NO | Not a public API endpoint (police-only) |

**Summary of gaps:** Yellow notice detail/images, UN notice type disambiguation (persons vs entities), UN notice detail endpoints. Green/Blue/Orange notices are police-only and not accessible via the public API — correct to omit.

---

### 2. OFAC (`intelligence_feeds/sanctions/ofac/endpoints.rs`)

**Base URL implemented:** `https://api.ofac-api.com/v4`

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| Screening | `POST /search` — search SDN/non-SDN list | YES | |
| Screening | `POST /screen` — screen name/entity via fuzzy logic | YES | |
| Sources | `GET /sources` — list available sanction sources | YES | |
| SDN List | `GET /sdn` — full Specially Designated Nationals list | YES | |
| Bulk Screening | `POST /screen/bulk` — batch screening of multiple entities | NO | Bulk endpoint exists in v4 |
| Downloadable Data | Consolidated SDN XML/CSV download | NO | Available at OFAC treasury.gov, not ofac-api.com |
| Screening Config | `GET /screen/config` — retrieve screening configuration | NO | May exist in enterprise tier |
| Historical | SDN historical changes/deltas endpoint | NO | Not documented in public API |

**Summary of gaps:** Bulk screening endpoint is the main practical gap. The rest are either enterprise-tier or not part of ofac-api.com (they belong to treasury.gov direct downloads). Coverage is good for core use-case.

---

### 3. OpenSanctions (`intelligence_feeds/sanctions/opensanctions/endpoints.rs`)

**Base URL implemented:** `https://api.opensanctions.org`

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| Search | `GET /search/{dataset}` — full-text entity search | YES | Implemented as `/search/default` |
| Match | `POST /match/{dataset}` — entity screening/matching | YES | Implemented as `/match/default` |
| Entity | `GET /entities/{entity_id}` — get entity by ID | YES | |
| Entity Adjacent | `GET /entities/{entity_id}/adjacent` — related entities (family, associates) | NO | Missing — important for relationship traversal |
| Entity Properties | `GET /entities/{entity_id}/properties/{prop}` — specific property | NO | Missing |
| Datasets | `GET /datasets/` — list all datasets | YES | |
| Dataset Detail | `GET /datasets/{name}` — specific dataset info | YES | |
| Collections | `GET /collections/` — list collections (sanctions, peps, etc.) | YES | |
| Health | `GET /healthz` — service health check | NO | Minor utility endpoint |
| Statistics | `GET /statistics` — entity/dataset statistics | NO | Useful for monitoring |
| Algorithms | `GET /algorithms` — list matching algorithms | NO | |
| Reconcile | `GET /reconcile/{dataset}` — OpenRefine reconciliation API | NO | Reconciliation API spec support |
| Statements | `GET /statements` — granular statement-level data | NO | Useful for provenance/audit |
| UpdateZ | `POST /updatez` — trigger data refresh (self-hosted only) | NO | Not applicable to hosted API |

**Summary of gaps:** Entity adjacency endpoint is the most significant omission for intelligence use. Health/statistics are operational utilities. Reconcile endpoint enables integration with data tooling like OpenRefine. `Entity adjacent` and `statements` endpoints are missing.

---

## Batch 29 — Corporate/Trade

---

### 4. GLEIF (`intelligence_feeds/corporate/gleif/endpoints.rs`)

**Base URL implemented:** `https://api.gleif.org/api/v1`

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| LEI Records | `GET /lei-records/{lei}` — get LEI record by code | YES | |
| LEI Records | `GET /lei-records` — search/filter by name | YES | Used for both name and country search |
| LEI Records | `GET /lei-records/{lei}/direct-parent` — direct parent | YES | |
| LEI Records | `GET /lei-records/{lei}/ultimate-parent` — ultimate parent | YES | |
| LEI Records | `GET /lei-records/{lei}/direct-children` — direct children | YES | |
| LEI Records | `GET /lei-records/{lei}/ultimate-children` — ultimate children | NO | Missing — useful for full subsidiary tree |
| Relationship Records | `GET /relationship-records` — search ownership relationships | NO | Level 2 ownership data, separate resource |
| Relationship Records | `GET /relationship-records/{lei}` — relationships for specific LEI | NO | Missing |
| ISIN Mapping | `GET /lei-records?filter[isin]={isin}` — lookup LEI by ISIN | NO | Filter parameter not modeled |
| BIC Mapping | `GET /bic-issuer-maps` — map BIC codes to LEI | NO | Separate resource endpoint |
| BIC Mapping | `GET /bic-issuer-maps/{bic}` — get LEI for specific BIC | NO | Missing |
| Registration Agents | `GET /registration-agents` — list authorized LEI issuers | NO | Reference data endpoint |
| Fuzzy Completions | `GET /fuzzy-completions` — autocomplete entity names | NO | Useful for search-as-you-type |
| Reporting Exceptions | `GET /reporting-exceptions` — entities exempt from parent disclosure | NO | Level 2 data for ownership exceptions |
| Reporting Exceptions | `GET /reporting-exceptions/{lei}` — exceptions for specific LEI | NO | Missing |

**Summary of gaps:** GLEIF has substantially more surface than implemented. Key missing endpoints: `relationship-records` (Level 2 ownership data — "who owns whom"), `bic-issuer-maps`, `reporting-exceptions`, `fuzzy-completions`, and `ultimate-children`. The current implementation covers only basic Level 1 LEI lookup and simple hierarchy traversal.

---

### 5. OpenCorporates (`intelligence_feeds/corporate/opencorporates/endpoints.rs`)

**Base URL implemented:** `https://api.opencorporates.com/v0.4`

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| Companies | `GET /companies/search` — search companies | YES | |
| Companies | `GET /companies/{jurisdiction}/{number}` — get company | YES | |
| Officers | `GET /officers/search` — search officers | YES | |
| Officers | `GET /officers/{id}` — get specific officer | NO | Direct officer lookup by ID missing |
| Company Officers | `GET /companies/{jurisdiction}/{number}/officers` | YES | |
| Company Filings | `GET /companies/{jurisdiction}/{number}/filings` | YES | |
| Company Statements | `GET /companies/{jurisdiction}/{number}/statements` — addresses, websites, ownership | NO | Replaces deprecated /data |
| Filings | `GET /filings/{id}` — specific filing detail | NO | Missing direct filing lookup |
| Statements | `GET /statements/{id}` — specific statement | NO | Missing |
| Control Statements | `GET /statements/control_statements/search` — ownership/control search | NO | Key for beneficial ownership chains |
| Subsequent Registrations | `GET /statements/subsequent_registrations/search` | NO | Missing |
| Alternate Registrations | `GET /statements/alternate_registrations/search` | NO | Missing |
| Gazette Notices | `GET /statements/gazette_notices/search` — official gazette entries | NO | Missing |
| Trademark Registrations | `GET /statements/trademark_registrations/search` | NO | Missing |
| Placeholders | `GET /placeholders/{id}` — probable unmatched companies | NO | Missing |
| Placeholders | `GET /placeholders/{id}/statements` | NO | Missing |
| Industry Codes | `GET /industry_codes` — list all classification schemes | NO | Reference data |
| Industry Codes | `GET /industry_codes/{scheme_id}` — codes in scheme | NO | |
| Industry Codes | `GET /industry_codes/{scheme_id}/{code}` — specific code | NO | |
| Jurisdictions | `GET /jurisdictions` — list all jurisdictions | YES | |
| Jurisdictions | `GET /jurisdictions/match` — match name to code | NO | Fuzzy jurisdiction matching |
| Corporate Groupings | `GET /corporate_groupings/search` — search groupings | YES | |
| Account Status | `GET /account_status` — API quota/usage | NO | Operational utility |

**Summary of gaps:** Significant coverage gap. The statements/control_statements endpoint is the most important missing item — it exposes beneficial ownership and control chains. Officer direct lookup, filing detail lookup, and the full statements subsystem are all absent. Industry codes are entirely missing.

---

### 6. UK Companies House (`intelligence_feeds/corporate/uk_companies_house/endpoints.rs`)

**Base URL implemented:** `https://api.company-information.service.gov.uk`

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| Search | `GET /search/companies` — search companies | YES | |
| Search | `GET /search` — search all records (companies + officers) | NO | Combined search missing |
| Search | `GET /search/officers` — search officers | NO | Missing |
| Search | `GET /search/disqualified-officers` — search disqualified officers | NO | Missing |
| Search | `GET /advanced-search/companies` — advanced search with more filters | NO | More powerful than basic search |
| Search | `GET /alphabetical-search/companies` — alphabetical company listing | NO | Missing |
| Search | `GET /dissolved-search/companies` — search dissolved companies | NO | Missing |
| Company | `GET /company/{number}` — company profile | YES | |
| Company | `GET /company/{number}/registered-office-address` — registered address only | NO | Separate address endpoint |
| Company | `GET /company/{number}/officers` — company officers | YES | |
| Company | `GET /company/{number}/appointments/{appointment_id}` — specific appointment | NO | Missing |
| Company | `GET /company/{number}/registers` — statutory registers | NO | Missing |
| Company | `GET /company/{number}/charges` — list charges/mortgages | YES | |
| Company | `GET /company/{number}/charges/{charge_id}` — specific charge | NO | Missing |
| Company | `GET /company/{number}/filing-history` — filing history list | YES | |
| Company | `GET /company/{number}/filing-history/{transaction_id}` — specific filing | NO | Missing |
| Company | `GET /company/{number}/insolvency` — insolvency info | YES | |
| Company | `GET /company/{number}/exemptions` — company exemptions | NO | Missing |
| Company | `GET /company/{number}/uk-establishments` — UK branch establishments | NO | Missing |
| PSC | `GET /company/{number}/persons-with-significant-control` — PSC list | YES | |
| PSC | `GET /company/{number}/persons-with-significant-control/{psc_id}` — specific PSC | NO | Missing |
| PSC | Corporate entity PSC, legal person PSC, super-secure PSC endpoints | NO | Multiple PSC sub-types missing |
| Officers | `GET /officers/{officer_id}/appointments` — officer's cross-company appointments | YES | |
| Disqualifications | `GET /disqualified-officers/natural/{officer_id}` — natural person disqualifications | NO | Missing |
| Disqualifications | `GET /disqualified-officers/corporate/{officer_id}` — corporate disqualifications | NO | Missing |

**Summary of gaps:** Substantial gaps. Search is underimplemented (6 missing variants). Missing specific-item endpoints for charges, filings, PSC. Missing officer disqualifications entirely. Missing company registers, exemptions, and UK-establishment endpoints.

---

### 7. Comtrade (`intelligence_feeds/trade/comtrade/endpoints.rs`)

**Base URL implemented:** `https://comtradeapi.un.org`

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| Data | `GET /data/v1/get/{typeCode}/{freqCode}/{clCode}` — authenticated trade data | YES | |
| Data | `GET /data/v1/preview/{typeCode}/{freqCode}/{clCode}` — preview (no auth) | YES | |
| Tariff-line | `GET /data/v1/get/tariffline/A/{clCode}` — tariff-line data (paid tier) | NO | More granular than commodity-level |
| Tariff-line | `GET /data/v1/preview/tariffline/A/{clCode}` — preview tariff-line | NO | Missing |
| Bulk | `GET /files/v1/getFinalData/{typeCode}/{freqCode}/{clCode}/{period}/{reporterCode}` — bulk download | NO | Bulk zip download endpoint |
| Bulk | `GET /files/v1/getOfficialData/...` — official bulk data | NO | Missing |
| Metadata | `GET /public/v1/getLOV/reporterCode` — reporters list | YES | |
| Metadata | `GET /public/v1/getLOV/partnerCode` — partners list | YES | |
| Metadata | `GET /public/v1/getLOV/cmdCode/{classification}` — commodity codes | YES | |
| Metadata | `GET /public/v1/getLOV/flowCode` — flow codes | YES | |
| Metadata | `GET /public/v1/getLOV/typeCode` — type codes | YES | |
| Metadata | `GET /public/v1/getLOV/freqCode` — frequency codes | YES | |
| Metadata | `GET /public/v1/getLOV/clCode` — classification codes | NO | Missing classification system list |
| Metadata | `GET /public/v1/getLOV/motCode` — mode of transport | NO | Missing |
| Metadata | `GET /public/v1/getLOV/customsCode` — customs procedure codes | NO | Missing |

**Summary of gaps:** Metadata is well covered for the primary use case. Main gaps are: tariff-line data (more granular trade data), bulk file download endpoints, and several minor metadata code lists (clCode, motCode, customsCode). Tariff-line access requires paid subscription.

---

### 8. EU TED (`intelligence_feeds/trade/eu_ted/endpoints.rs`)

**Base URL implemented:** `https://ted.europa.eu/api/v3.0`

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| Notices | `POST /notices/search` — search procurement notices | YES | |
| Notices | `GET /notices/{notice_id}` — get specific notice | YES | |
| Business Entities | `POST /business-entities/search` — search entities | YES | |
| Business Entities | `GET /business-entities/{entity_id}` — entity detail | YES | |
| Codelists | `GET /codelists/{codelist_id}` — get codelist values | YES | |
| Codelists | `GET /codelists` — list all available codelists | NO | Index of all codelists missing |
| eForms | `POST /eforms/notices` — submit eForms notice (eSender) | NO | Write/submit endpoint — not needed for read-only intelligence feed |
| eForms | `GET /eforms/notices/{notice_id}` — eForms notice detail | NO | eForms-specific format |
| Statistics | `GET /notices/statistics` — procurement volume stats | NO | Aggregate stats endpoint |
| Latest | `GET /notices/latest` — most recently published notices | NO | Convenience endpoint |
| Documents | `GET /notices/{id}/documents` — attached procurement documents | NO | Supporting documents |
| Validation | `POST /notices/validate` — validate eForms (eSender only) | NO | eSender workflow — not applicable |

**Summary of gaps:** Core read-only functionality is well covered. Main gap is the codelists index endpoint. eForms submission/validation endpoints are eSender-specific and out of scope for an intelligence feed. Statistics and latest-notices convenience endpoints are useful additions.

---

## Batch 30 — Demographics/Governance/News

---

### 9. UN OCHA (`intelligence_feeds/demographics/un_ocha/endpoints.rs`)

**Base URL implemented:** `https://hapi.humdata.org/api/v1`

> **Note:** The API has migrated from v1 to v2 (`/api/v2/`). The implemented base path is outdated.

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| Population | `GET /api/v2/affected-people/...` (v2 scheme) | PARTIAL | v1 path used, v2 is current |
| Population | `GET /population` (v1) | YES | Path correct for v1 |
| Food Security | `GET /food-security` (v1) | YES | |
| Humanitarian Needs | `GET /humanitarian-needs` (v1) | YES | |
| Operational Presence | `GET /operational-presence` (v1) | YES | |
| Funding | `GET /funding` (v1) | YES | |
| Refugees | `GET /refugees` (v1) | YES | Maps to v2 `affected-people/refugees-persons-of-concern` |
| IDPs | `GET /idps` (v1) | YES | |
| Returnees | `GET /returnees` (v1) | YES | |
| Conflict Events | `GET /coordination-context/conflict-events` | NO | ACLED armed conflict data — new in v2 |
| National Risk | `GET /coordination-context/national-risk` | NO | INFORM risk framework — new in v2 |
| Food Prices | `GET /food-security-nutrition-poverty/food-prices-market-monitor` | NO | WFP commodity prices — new in v2 |
| Encode App ID | `GET /encode_app_identifier` — generate base64 client identifier | NO | Auth utility for generating app_identifier |
| Metadata | Various dataset/location metadata endpoints | NO | Coverage metadata endpoints |

**Summary of gaps:** The API has migrated to v2 — base URL should be updated to `/api/v2`. Key additions in v2 include conflict events (ACLED), national risk (INFORM), and food prices (WFP) which are all missing. The `encode_app_identifier` endpoint is needed for API access (app_identifier required in requests).

---

### 10. UN Population (`intelligence_feeds/demographics/un_population/endpoints.rs`)

**Base URL implemented:** `https://population.un.org/dataportalapi/api/v1`

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| Locations | `GET /locations` — list all locations | YES | |
| Locations | `GET /locations/{codes}` — filter by ISO/M49 codes | NO | Parameterized lookup missing |
| Locations | `GET /locationsWithAggregates` — locations + aggregate regions | NO | Missing |
| Indicators | `GET /Indicators` — list all indicators | YES | |
| Indicators | `GET /Indicators/{codes}` — filter by ID or short name | YES | Via IndicatorDetails |
| Data | `GET /data/indicators/{indicators}/locations/{locations}` — flexible data query | NO | Main data endpoint missing |
| Data | `GET /data/indicators/{indicators}/locations/{locations}/start/{y}/end/{y}` | NO | Year-range data query missing |
| Empirical Data | `GET /empirical/data` — empirical (observed) data only | NO | Distinct from projected data |
| Dimensions | `GET /dimensions/times/{timeType}` — available time values | NO | Missing |
| Dimensions | `GET /dimensions/variants` — projection variants (medium, high, low) | NO | Missing |
| Metadata | `GET /metadata/ages/{indicatorIds}` — age metadata | NO | Missing |
| Metadata | `GET /metadata/sexes/{indicatorIds}` — sex disaggregation metadata | NO | Missing |
| Metadata | `GET /metadata/categories/{indicatorIds}` — category metadata | NO | Missing |
| Sources | `GET /sources` — list all data sources | NO | Missing |
| Sources | `GET /sources/{ids}/indicators` — indicators for a source | NO | Missing |
| Topics | `GET /topics` — list data topics | NO | Missing |
| Auth | `POST /token/request` — request bearer token via email | NO | Required for data access |

**Summary of gaps:** Significant coverage gap. The current implementation covers only basic listing endpoints (locations, indicators) but is entirely missing the actual data retrieval endpoints (`/data/indicators/{...}/locations/{...}`). The data endpoint is the core of the API. Also missing: authentication token flow, empirical data, dimensions, metadata disaggregations, sources, and topics.

---

### 11. WHO (`intelligence_feeds/demographics/who/endpoints.rs`)

**Base URL implemented:** `https://ghoapi.azureedge.net/api`

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| Indicators | `GET /Indicator` — list all indicators | YES | |
| Indicators | `GET /Indicator?$filter=...` — filter indicators by name | NO | OData filter not modeled as endpoint |
| Indicator Data | `GET /{INDICATOR_CODE}` — data for specific indicator | YES | |
| Indicator Data | `GET /{INDICATOR_CODE}?$filter=...` — filtered indicator data | NO | OData filtering not modeled |
| Dimensions | `GET /DIMENSION/COUNTRY` — list of countries | YES | |
| Dimensions | `GET /DIMENSION/REGION` — list of regions | YES | |
| Dimensions | `GET /DIMENSION/{CODE}/DimensionValues` — values for any dimension | NO | Generic dimension lookup missing |
| Dimensions | `GET /Dimension` — list of all available dimension codes | NO | Dimension catalog missing |

**Summary of gaps:** Core data access is functional. The main gaps are: (1) the generic `DIMENSION/{CODE}/DimensionValues` endpoint for non-country/region dimensions (e.g., sex, age groups, income levels), (2) the `Dimension` catalog endpoint listing all available dimension codes, and (3) OData `$filter` querying is not modeled as a typed construct. These are moderate gaps.

---

### 12. Wikipedia (`intelligence_feeds/demographics/wikipedia/endpoints.rs`)

**Base URL implemented:** `https://wikimedia.org/api/rest_v1/metrics/pageviews`

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| Pageviews | `GET /per-article/{project}/{access}/{agent}/{article}/{granularity}/{start}/{end}` | YES | |
| Pageviews | `GET /aggregate/{project}/{access}/{agent}/{granularity}/{start}/{end}` | YES | |
| Pageviews | `GET /top/{project}/{access}/{year}/{month}/{day}` — top articles | YES | |
| Pageviews | `GET /top-per-country/{project}/{access}/{year}/{month}` — pageviews by country | YES | |
| Pageviews | `GET /top-by-country/{country}/{access}/{year}/{month}` — top articles for a country | NO | Country-specific top articles |
| Unique Devices | `GET /metrics/unique-devices/{project}/{access-site}/{granularity}/{start}/{end}` | NO | Unique device counts — separate base path |
| Edited Pages | `GET /metrics/edited-pages/aggregate/{project}/{editor-type}/{page-type}/{granularity}/{start}/{end}` | NO | Edit analytics — separate base path |
| Edits | `GET /metrics/edits/aggregate/{project}/{editor-type}/{page-type}/{granularity}/{start}/{end}` | NO | Raw edit counts |
| Editors | `GET /metrics/editors/aggregate/{project}/{editor-type}/{page-type}/{granularity}/{start}/{end}` | NO | Editor counts |
| Registered Users | `GET /metrics/registered-users/new/{project}/{granularity}/{start}/{end}` | NO | New user registrations |
| Bytes Diff | `GET /metrics/bytes-difference/absolute/aggregate/{project}/{...}` | NO | Content change volume |
| Media Requests | `GET /metrics/media/aggregate/{project}/{...}` | NO | Media file access counts |

**Summary of gaps:** The implementation covers only the pageviews subsystem. The Wikimedia Analytics API has five additional major subsystems all using the same `wikimedia.org/api/rest_v1/metrics/` base but different sub-paths: unique-devices, edited-pages, edits, editors, and registered-users. The base URL in the connector is artificially restricted to `/metrics/pageviews` — it should be `/metrics` to accommodate the full API surface.

---

### 13. EU Parliament (`intelligence_feeds/governance/eu_parliament/endpoints.rs`)

**Base URL implemented:** `https://data.europarl.europa.eu/api/v1`

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| MEPs | `GET /meps` — list MEPs | YES | |
| MEPs | `GET /meps/{id}` — MEP by ID | YES | |
| MEPs | `GET /meps/{id}/activities` — MEP activity list | NO | Missing |
| Plenary Documents | `GET /plenary-documents` — list documents | YES | |
| Plenary Documents | `GET /plenary-documents/{id}` — document by ID | YES | |
| Meetings | `GET /meetings` — list plenary sessions | YES | |
| Meetings | `GET /meetings/{id}` — specific meeting | NO | Missing |
| Committees | `GET /committees` — list committees | YES | |
| Committees | `GET /committees/{id}` — specific committee | NO | Missing |
| Votes | `GET /vote-results` — roll-call vote results | NO | Key governance data — entirely missing |
| Votes | `GET /vote-results/{id}` — specific vote | NO | Missing |
| Parliamentary Questions | `GET /parliamentary-questions` — oral/written questions | NO | Entirely missing |
| Parliamentary Questions | `GET /parliamentary-questions/{id}` — specific question | NO | Missing |
| Activities | `GET /activities` — MEP parliamentary activities | NO | Missing |
| Adopted Texts | `GET /adopted-texts` — adopted legislative texts | NO | Missing |

**Summary of gaps:** Significant gaps. The most important missing endpoint is `vote-results` — roll-call votes are the primary use-case for parliamentary intelligence. Also missing: parliamentary questions, activities, and adopted texts. Individual resource detail endpoints (meetings/{id}, committees/{id}) are also absent.

---

### 14. UK Parliament (`intelligence_feeds/governance/uk_parliament/endpoints.rs`)

**Base URL:** `https://members-api.parliament.uk/api` + `https://bills-api.parliament.uk/api/v1`

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| Members | `GET /Members/Search` — search members | YES | |
| Members | `GET /Members/{id}` — member detail | YES | |
| Members | `GET /Members/{id}/Voting` — voting record | YES | |
| Members | `GET /Members/SearchHistorical` — historical members | NO | Missing |
| Members | `GET /Members/{id}/Biography` — member career history | NO | Missing |
| Members | `GET /Members/{id}/Contact` — contact info | NO | Missing |
| Members | `GET /Members/{id}/ContributionSummary` — parliamentary contributions | NO | Missing |
| Members | `GET /Members/{id}/Edms` — early day motions | NO | Missing |
| Members | `GET /Members/{id}/Experience` — professional background | NO | Missing |
| Members | `GET /Members/{id}/Focus` — policy interests | NO | Missing |
| Members | `GET /Members/{id}/LatestElectionResult` — election performance | NO | Missing |
| Members | `GET /Members/{id}/RegisteredInterests` — financial interests | NO | Key for transparency/intelligence |
| Members | `GET /Members/{id}/WrittenQuestions` — parliamentary questions | NO | Missing |
| Members | `GET /Members/History` — historical data by IDs | NO | Missing |
| Lords Interests | `GET /LordsInterests/Register` — Lords registered interests | NO | Missing |
| Parties | `GET /Parties/StateOfTheParties/{house}/{forDate}` — seat distribution | NO | Missing |
| Parties | `GET /Parties/LordsByType/{forDate}` — Lords composition | NO | Missing |
| Parties | `GET /Parties/GetActive/{house}` — active parties | NO | Missing |
| Posts | `GET /Posts/GovernmentPosts` — government positions | NO | Missing |
| Posts | `GET /Posts/OppositionPosts` — opposition roles | NO | Missing |
| Constituencies | `GET /Location/Constituency/Search` — search constituencies | YES | |
| Constituencies | `GET /Location/Constituency/{id}` — constituency detail | NO | Missing |
| Constituencies | `GET /Location/Constituency/{id}/ElectionResults` — election results | NO | Missing |
| Bills | `GET /Bills` — list bills | YES | |
| Bills | `GET /Bills/{id}` — bill detail | YES | |
| Bills | `GET /Bills/{id}/Stages` — bill legislative stages | YES | |
| Bills | `GET /Bills/{id}/Stages/{stageId}` — specific stage detail | NO | Missing |
| Bills | `GET /Bills/{id}/Stages/{stageId}/Amendments` — amendments | NO | Missing |
| Bills | `GET /Bills/{id}/Stages/{stageId}/Amendments/{amendmentId}` — specific amendment | NO | Missing |
| Bills | `GET /Bills/{id}/Publications` — bill publications | NO | Missing |
| Bills | `GET /Bills/{id}/NewsArticles` — news articles for bill | NO | Missing |
| Bills | `GET /Bills/{id}/Stages/{stageId}/PingPongItems` — ping pong motions | NO | Missing |
| Bills | `GET /BillTypes` — list bill types | NO | Missing |
| Bills | `GET /PublicationTypes` — list publication types | NO | Missing |
| Bills | `GET /Stages` — all bill stages reference data | NO | Missing |
| Bills | `GET /Sittings` — list parliamentary sittings | NO | Missing |
| Bills RSS | `GET /Rss/allbills.rss`, `publicbills.rss`, `privatebills.rss` | NO | RSS feed convenience endpoints |
| Reference | `GET /Reference/PolicyInterests` — policy interest categories | NO | Missing |
| Reference | `GET /Reference/Departments` — department information | NO | Missing |

**Summary of gaps:** The implementation covers only ~10% of the available Members API and ~40% of the Bills API. Key missing items: `RegisteredInterests` (critical for financial transparency intelligence), `ContributionSummary`, historical member search, parties composition, posts (government/opposition), full amendment tracking, and all reference data endpoints.

---

### 15. Hacker News (`intelligence_feeds/hacker_news/endpoints.rs`)

**Base URL implemented:** `https://hacker-news.firebaseio.com/v0`

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| Stories | `GET /topstories.json` — top 500 stories | YES | |
| Stories | `GET /newstories.json` — newest 500 stories | YES | |
| Stories | `GET /beststories.json` — best 500 stories | YES | |
| Stories | `GET /askstories.json` — Ask HN up to 200 | YES | |
| Stories | `GET /showstories.json` — Show HN up to 200 | YES | |
| Stories | `GET /jobstories.json` — job posts up to 200 | YES | |
| Items | `GET /item/{id}.json` — any item (story/comment/job/poll) | YES | |
| Users | `GET /user/{id}.json` — user profile | YES | |
| Max Item | `GET /maxitem.json` — current max item ID | YES | |
| Updates | `GET /updates.json` — recently changed items and profiles | YES | |
| Firebase Streaming | `wss://{url}?print=pretty` — real-time Server-Sent Events | NO | Firebase SSE streaming not modeled (optional) |

**Summary of gaps:** Coverage is complete for the REST API. The only notable omission is Firebase's real-time streaming (Server-Sent Events/SSE) capability which enables push notifications when the top-stories list changes — this could be added as an optional `ws_base` for a live feed mode.

---

### 16. RSS Proxy (`intelligence_feeds/rss_proxy/endpoints.rs`)

**Note:** RSS Proxy is fundamentally a URL-passthrough connector, not a structured REST API.

| Category | Endpoint | We Have? | Notes |
|---|---|---|---|
| Feed URL | `Feed { url: String }` — direct URL fetch | YES | Core mechanism |
| News Sources | BBC World, Technology, Business | YES | |
| News Sources | Reuters World, Business, Technology | YES | |
| News Sources | NPR News, Business, Technology | YES | |
| News Sources | Guardian World, Business, Technology | YES | |
| News Sources | Al Jazeera | YES | |
| News Sources | CNN Top, World, Business | YES | |
| Tech Sources | TechCrunch, Ars Technica, The Verge, Wired, MIT Tech Review, ZDNet | YES | |
| Policy Sources | CSIS, Brookings, Carnegie, CFR, RAND, War on the Rocks | YES | |
| Finance Sources | FT, Bloomberg, MarketWatch, Seeking Alpha | YES | |
| Cyber Sources | Krebs, Schneier, Dark Reading, The Hacker News | YES | |
| Finance Sources | WSJ Markets, Economist | NO | Common sources not included |
| Finance Sources | CNBC Top News, Crypto | NO | Missing |
| Crypto News | CoinDesk, CoinTelegraph, Decrypt | NO | Crypto-specific feeds missing |
| Government Sources | US Federal Register, UK Gov, EU EUR-Lex | NO | Regulatory feeds missing |
| Sanctions/Legal | OFAC SDN updates RSS, UN sanctions list updates | NO | Sanctions-specific feeds missing |
| Science | Nature, Science, arXiv | NO | Academic/science feeds missing |
| Aggregator utility | `POST /rss2json` or proxy parsing service | NO | Not applicable — direct fetch is correct |

**Summary of gaps:** The architectural design is sound and complete. The gaps are purely in the catalogue of pre-configured feed URLs. Missing notable categories: crypto news feeds (CoinDesk, CoinTelegraph), government/regulatory RSS (Federal Register, EUR-Lex), academic science feeds, and dedicated sanctions-update feeds.

---

## Cross-Connector Priority Summary

| Priority | Connector | Gap | Effort |
|---|---|---|---|
| HIGH | UN Population | Missing core `/data/` endpoint — effectively non-functional for data retrieval | Low |
| HIGH | UN OCHA | v1 → v2 API migration needed; missing conflict events and food prices | Low |
| HIGH | OpenCorporates | `control_statements/search` — beneficial ownership chains missing | Low |
| HIGH | EU Parliament | `vote-results` endpoint entirely missing | Low |
| HIGH | UK Parliament | `RegisteredInterests` and full amendment tracking missing | Medium |
| HIGH | GLEIF | `relationship-records` (Level 2 ownership) entirely missing | Low |
| MEDIUM | OpenSanctions | Entity adjacency endpoint missing | Low |
| MEDIUM | UK Companies House | 6 search variants missing; disqualifications missing | Low |
| MEDIUM | Wikipedia | 4 analytics subsystems missing (unique-devices, edits, editors, registered-users) | Low |
| MEDIUM | INTERPOL | Yellow notice details; UN notice disambiguation | Low |
| LOW | Comtrade | Tariff-line and bulk endpoints (paid tier) | Low |
| LOW | EU TED | Codelists index; statistics endpoint | Low |
| LOW | WHO | Generic dimension endpoint; Dimension catalog | Low |
| LOW | OFAC | Bulk screening endpoint | Low |
| LOW | Hacker News | Firebase SSE streaming (optional) | Low |
| MINIMAL | RSS Proxy | Additional feed URL catalogue entries | Low |

---

## Sources

- [INTERPOL Public Notices API — interpol.api.bund.dev](https://interpol.api.bund.dev/)
- [INTERPOL Notices — ws-public.interpol.int](https://ws-public.interpol.int/notices/v1/red)
- [OFAC API Documentation — docs.ofac-api.com](https://docs.ofac-api.com/)
- [OpenSanctions API Documentation](https://www.opensanctions.org/docs/api/)
- [OpenSanctions yente GitHub](https://github.com/opensanctions/yente)
- [GLEIF API Documentation](https://www.gleif.org/en/lei-data/gleif-api)
- [GLEIF API Changes Documentation](https://www.gleif.org/content/4_lei-data/1_access-and-use-lei-data/6_supporting-documents/GLEIF-API-Changes-Documentation.html)
- [OpenCorporates API Reference v0.4.8](https://api.opencorporates.com/documentation/API-Reference)
- [UK Companies House API Specification Summary](https://developer-specs.company-information.service.gov.uk/companies-house-public-data-api/reference)
- [UK Companies House Developer Overview](https://developer.company-information.service.gov.uk/overview)
- [UN COMTRADE Developer Portal](https://comtradedeveloper.un.org/)
- [comtradeapicall Python library — GitHub](https://github.com/uncomtrade/comtradeapicall)
- [EU TED API Documentation](https://docs.ted.europa.eu/api/latest/index.html)
- [HDX HAPI OpenAPI Documentation](https://hapi.humdata.org/docs)
- [UN Population Data Portal API](https://population.un.org/dataportalapi/index.html)
- [WHO GHO OData API Documentation](https://www.who.int/data/gho/info/gho-odata-api)
- [Wikimedia Analytics API](https://doc.wikimedia.org/generated-data-platform/aqs/analytics-api/)
- [EU Parliament Open Data Portal — Developer Corner](https://data.europarl.europa.eu/en/developer-corner/opendata-api)
- [UK Parliament Members API](https://members-api.parliament.uk/)
- [UK Parliament Bills API](https://bills-api.parliament.uk/)
- [UK Parliament Developer Hub](https://developer.parliament.uk/)
- [Hacker News API — GitHub](https://github.com/HackerNews/API)
