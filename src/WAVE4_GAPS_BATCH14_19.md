# Wave 4 Endpoint Gaps — Batches 14–19

> Generated: 2026-03-13
> Base path: `digdigdig3/src/`
> Method: Read each `endpoints.rs`, then cross-checked against official API documentation.

---

## Batch 14 — Economic (Central Banks)

### 1. BIS — `intelligence_feeds/economic/bis/endpoints.rs`

Official API: `https://stats.bis.org/api/v2`
Documentation: https://data.bis.org/help/tools

**What we have:** Data, DataAll, Dataflows, Dataflow, DataStructure, Codelist, ConceptScheme, Availability (8 variants total)

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Data | `GET /data/dataflow/BIS/{flow}/latest/{key}` | YES | `Data` variant |
| Data | `GET /data/dataflow/BIS/{flow}/latest/all` | YES | `DataAll` variant |
| Structure | `GET /structure/dataflow/BIS` | YES | `Dataflows` |
| Structure | `GET /structure/dataflow/BIS/{id}/latest` | YES | `Dataflow` |
| Structure | `GET /structure/datastructure/BIS/{id}` | YES | `DataStructure` |
| Structure | `GET /structure/codelist/BIS/{id}` | YES | `Codelist` |
| Structure | `GET /structure/conceptscheme/BIS/{id}` | YES | `ConceptScheme` |
| Availability | `GET /availability/dataflow/BIS/{flow}/latest/{key}` | YES | `Availability` |
| Structure | `GET /structure/categoryscheme/BIS` | **NO** | Category scheme — groups related dataflows |
| Structure | `GET /structure/categorisation/BIS` | **NO** | Links dataflows to categories |
| Structure | `GET /structure/contentconstraint/BIS/{id}` | **NO** | Content constraints for a DSD |
| Structure | `GET /structure/agencyscheme` | **NO** | Agency metadata |

**Gap count: 4 minor structural endpoints.** Core data access is complete.

---

### 2. BOE — `intelligence_feeds/economic/boe/endpoints.rs`

Official API: `https://www.bankofengland.co.uk/boeapps/database`
Documentation: https://www.bankofengland.co.uk/boeapps/database/help.asp

**What we have:** GetData (`/_iadb-fromshowcolumns.asp`), GetSeriesInfo (`/fromshowcolumns.asp`) — 2 endpoints total

**Note:** The BoE IADB is a legacy CSV-download interface, not a true REST API. There is no published OpenAPI spec. The two paths we have cover essentially the entire machine-accessible surface:

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Data | `/_iadb-fromshowcolumns.asp` (CSV export, date-ranged) | YES | `GetData` |
| Data | `/fromshowcolumns.asp` (series info/CSV) | YES | `GetSeriesInfo` |
| Browse | `/fromweb.asp?SectionRequired={cat}` | **NO** | Browse category pages (HTML only, not machine-usable) |
| Metadata | Series list / search (no public REST endpoint exists) | **NO** | No official search API — requires scraping or knowing codes |

**Gap count: 2 — but these are not real REST endpoints.** The BoE database does not expose a proper REST API for series discovery; series codes must be obtained from publications. Coverage is complete for what is programmatically stable.

---

### 3. Bundesbank — `intelligence_feeds/economic/bundesbank/endpoints.rs`

Official API: `https://api.statistiken.bundesbank.de/rest`
Documentation: https://www.bundesbank.de/en/statistics/time-series-databases/help-for-sdmx-web-service

**What we have:** Data, DataByTsId, ListDataflows, Dataflow, DataStructure, Codelist, ConceptScheme, Metadata — 8 variants

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Data | `GET /data/{dataflow}/{key}` | YES | `Data` |
| Data | `POST /data/tsIdList` | YES | `DataByTsId` |
| Structure | `GET /dataflow/BBK` | YES | `ListDataflows` |
| Structure | `GET /dataflow/BBK/{id}` | YES | `Dataflow` |
| Structure | `GET /datastructure/BBK/{id}` | YES | `DataStructure` |
| Structure | `GET /codelist/BBK/{id}` | YES | `Codelist` |
| Structure | `GET /conceptscheme/BBK/{id}` | YES | `ConceptScheme` |
| Metadata | `GET /metadata/dataflow/BBK/{flow}/{key}` | YES | `Metadata` |
| Structure | `GET /categoryscheme/BBK` | **NO** | Category groupings |
| Structure | `GET /categorisation/BBK` | **NO** | Dataflow-to-category links |
| Data | `GET /data/{dataflow}/{key}?updatedAfter=` | **NO** | Delta fetch by update timestamp (SDMX feature) |

**Gap count: 3 minor SDMX structural endpoints.** Core coverage is complete.

---

### 4. CBR — `intelligence_feeds/economic/cbr/endpoints.rs`

Official API: `https://www.cbr.ru`
Documentation: https://www.cbr.ru/development/

**What we have:** KeyRate, DailyJson, DailyXml, CurrencyList, ExchangeRateDynamic, MetalPrices, RepoRates, InternationalReserves, MonetaryBase, InterbankRates — 10 endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| JSON API | `GET /api/v1/press/keyrate` | YES | `KeyRate` |
| JSON API | `GET /api/v1/daily_json` | YES | `DailyJson` |
| XML API | `GET /scripts/XML_daily.asp` | YES | `DailyXml` |
| XML API | `GET /scripts/XML_val.asp` | YES | `CurrencyList` |
| XML API | `GET /scripts/XML_dynamic.asp` | YES | `ExchangeRateDynamic` |
| XML API | `GET /scripts/xml_metall.asp` | YES | `MetalPrices` |
| XML API | `GET /scripts/XML_repo.asp` | YES | `RepoRates` |
| XML API | `GET /scripts/XML_ostat.asp` | YES | `InternationalReserves` |
| XML API | `GET /scripts/XML_bic.asp` | YES | `MonetaryBase` |
| XML API | `GET /scripts/XML_mkr.asp` | YES | `InterbankRates` |
| XML API | `GET /scripts/XML_val.asp?Seld=0` | **NO** | Full currency list including obsolete |
| XML API | `GET /scripts/XML_Ruonia.asp` | **NO** | RUONIA overnight rate |
| XML API | `GET /scripts/XML_RO.asp` | **NO** | Refinancing rate history |
| XML API | `GET /scripts/XML_DepRates.asp` | **NO** | Deposit rates |
| XML API | `GET /scripts/XML_creditorgrate.asp` | **NO** | Creditor country rates |
| JSON API | `GET /api/v1/press/` (index) | **NO** | Press release data index |

**Gap count: 6 XML/JSON endpoints for specialty rates.** Key rate and FX are fully covered; RUONIA and deposit/refinancing rates are missing.

---

### 5. ECB — `intelligence_feeds/economic/ecb/endpoints.rs`

Official API: `https://data-api.ecb.europa.eu/service`
Documentation: https://data.ecb.europa.eu/help/api/overview

**What we have:** Data, Dataflows, Dataflow, DataStructure, CodeList, ConceptScheme — 6 variants

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Data | `GET /data/{dataflow}/{key}` | YES | `Data` |
| Structure | `GET /dataflow/ECB` | YES | `Dataflows` |
| Structure | `GET /dataflow/ECB/{id}/latest` | YES | `Dataflow` |
| Structure | `GET /datastructure/ECB/{id}` | YES | `DataStructure` |
| Structure | `GET /codelist/ECB/{id}` | YES | `CodeList` |
| Structure | `GET /conceptscheme/ECB/{id}` | YES | `ConceptScheme` |
| Structure | `GET /categoryscheme/ECB` | **NO** | Top-level category groupings |
| Structure | `GET /categorisation/ECB` | **NO** | Maps dataflows to categories |
| Structure | `GET /contentconstraint/ECB/{id}` | **NO** | Content constraint definitions |
| Availability | `GET /availableconstraint/{dataflow}/{key}` | **NO** | Distinct series keys for a query |

**Gap count: 4 structural endpoints.** Core data retrieval is complete; missing are the discovery/browsing metadata endpoints.

---

## Batch 15 — Economic (International)

### 6. DBnomics — `intelligence_feeds/economic/dbnomics/endpoints.rs`

Official API: `https://api.db.nomics.world/v22`
Documentation: https://docs.db.nomics.world/web-api/

**What we have:** Providers, Provider, Datasets, Dataset, SearchDatasets, Series, SeriesList, SearchSeries, ConvertSeriesId, LastUpdates — 10 endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Providers | `GET /providers` | YES | `Providers` |
| Providers | `GET /providers/{code}` | YES | `Provider` |
| Datasets | `GET /datasets/{provider}` | YES | `Datasets` |
| Datasets | `GET /datasets/{provider}/{dataset}` | YES | `Dataset` |
| Datasets | `GET /search/datasets?q=` | YES | `SearchDatasets` |
| Series | `GET /series/{provider}/{dataset}/{series}` | YES | `Series` |
| Series | `GET /series/{provider}/{dataset}` | YES | `SeriesList` |
| Series | `GET /search/series?q=` | YES | `SearchSeries` |
| Series | `GET /series?series_ids=` | YES | `ConvertSeriesId` |
| Updates | `GET /last-updates` | YES | `LastUpdates` |
| Series | `GET /series?ids=&observations=1` | **NO** | Batch fetch with inline observations (query param variant) |
| Datasets | `GET /datasets/{provider}/{dataset}?align_periods=1` | **NO** | Period-aligned dataset fetch |

**Gap count: 2 — query-parameter variants of existing endpoints,** not truly new endpoints. Coverage is functionally complete.

---

### 7. ECOS — `intelligence_feeds/economic/ecos/endpoints.rs`

Official API: `https://ecos.bok.or.kr/api`
Documentation: https://ecos.bok.or.kr/api/

**What we have:** StatisticSearch, KeyStatisticList, StatisticTableList, StatisticItemList, StatisticWord, StatMeta — 6 endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Data | `/{key}/{format}/{lang}/StatisticSearch/{stat_code}/...` | YES | `StatisticSearch` |
| Metadata | `/{key}/{format}/{lang}/KeyStatisticList` | YES | `KeyStatisticList` |
| Metadata | `/{key}/{format}/{lang}/StatisticTableList/{stat_code}` | YES | `StatisticTableList` |
| Metadata | `/{key}/{format}/{lang}/StatisticItemList/{stat_code}` | YES | `StatisticItemList` |
| Search | `/{key}/{format}/{lang}/StatisticWord/{word}` | YES | `StatisticWord` |
| Metadata | `/{key}/{format}/{lang}/StatMeta/{data_name}` | YES | `StatMeta` |

**Gap count: 0.** The ECOS API has exactly 6 services and all are implemented. Coverage is complete.

---

### 8. Eurostat — `intelligence_feeds/economic/eurostat/endpoints.rs`

Official API: Multiple bases — statistics, SDMX, catalogue
Documentation: https://ec.europa.eu/eurostat/web/user-guides/data-browser/api-data-access/api-introduction

**What we have:** Data, Label (Statistics API); ListDataflows, Dataflow, DataSdmx, Datastructure, Codelist, ConceptScheme (SDMX API); TableOfContents (Catalogue API) — 9 endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Statistics | `GET /statistics/1.0/data/{dataset}` | YES | `Data` |
| Statistics | `GET /statistics/1.0/label/{dataset}` | YES | `Label` |
| SDMX | `GET /sdmx/2.1/dataflow/ESTAT/all/latest` | YES | `ListDataflows` |
| SDMX | `GET /sdmx/2.1/dataflow/ESTAT/{id}/latest` | YES | `Dataflow` |
| SDMX | `GET /sdmx/2.1/data/{flow}/{key}` | YES | `DataSdmx` |
| SDMX | `GET /sdmx/2.1/datastructure/ESTAT/{id}` | YES | `Datastructure` |
| SDMX | `GET /sdmx/2.1/codelist/ESTAT/{id}` | YES | `Codelist` |
| SDMX | `GET /sdmx/2.1/conceptscheme/ESTAT/{id}` | YES | `ConceptScheme` |
| Catalogue | `GET /catalogue/toc` (XML or TXT) | YES | `TableOfContents` |
| Catalogue | `GET /catalogue/dcat/ESTAT/FULL` | **NO** | DCAT-AP full catalogue for data.europa.eu |
| Catalogue | `GET /catalogue/dcat/ESTAT/UPDATES` | **NO** | DCAT-AP incremental updates |
| Catalogue | `GET /catalogue/rss/{lang}/statistics-update.rss` | **NO** | RSS feed of dataset updates |
| Catalogue | `GET /catalogue/metabase.txt.gz` | **NO** | Full metabase file (all dataset structures) |

**Gap count: 4 catalogue endpoints.** For a trading intelligence system the DCAT-AP and RSS feeds are low priority, but the metabase bulk file is useful for discovering new datasets.

---

### 9. FRED — `intelligence_feeds/economic/fred/endpoints.rs`

Official API: `https://api.stlouisfed.org`
Documentation: https://fred.stlouisfed.org/docs/api/fred/

**What we have:** 32 endpoints across Category (6), Release (9), Series (10), Source (3), Tag (3), GeoFRED (4) groups

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Category (6) | `/fred/category` through `/fred/category/related_tags` | YES | All 6 present |
| Release (9) | `/fred/releases` through `/fred/release/tables` | YES | All 9 present |
| Series (10) | `/fred/series` through `/fred/series/vintagedates` | YES | All 10 present |
| Source (3) | `/fred/sources` through `/fred/source/releases` | YES | All 3 present |
| Tag (3) | `/fred/tags` through `/fred/tags/series` | YES | All 3 present |
| GeoFRED (4) | `/geofred/series/group` through `/geofred/shapes/file` | YES | All 4 present |

**Gap count: 0.** FRED coverage is complete — all documented endpoints are implemented.

---

### 10. IMF — `intelligence_feeds/economic/imf/endpoints.rs`

Official API: `http://dataservices.imf.org/REST/SDMX_JSON.svc`
Also available: IMF DataMapper API at `https://www.imf.org/external/datamapper/api/`
Documentation: http://datahelp.imf.org/knowledgebase/articles/667681

**What we have:** Dataflow, CompactData, DataStructure, CodeList, GenericData — 5 endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| SDMX JSON | `GET /Dataflow` | YES | `Dataflow` |
| SDMX JSON | `GET /CompactData/{db}/{dims}` | YES | `CompactData` |
| SDMX JSON | `GET /DataStructure/{db}` | YES | `DataStructure` |
| SDMX JSON | `GET /CodeList/{list}_{db}` | YES | `CodeList` |
| SDMX JSON | `GET /GenericData/{db}/{dims}` | YES | `GenericData` |
| DataMapper | `GET /datamapper/api/v1/indicators` | **NO** | All WEO indicators list |
| DataMapper | `GET /datamapper/api/v1/countries` | **NO** | Countries list |
| DataMapper | `GET /datamapper/api/v1/regions` | **NO** | Regions/analytical groups |
| DataMapper | `GET /datamapper/api/v1/data/{indicator}` | **NO** | WEO time series by indicator |
| SDMX | `GET /GetDataSetList` (alternate discovery) | **NO** | Dataset list (some IMF apps use this path) |

**Gap count: 5 — the DataMapper API is a separate API not covered at all.** It provides direct access to WEO (World Economic Outlook) forecasts which are highly relevant for macroeconomic intelligence.

---

### 11. OECD — `intelligence_feeds/economic/oecd/endpoints.rs`

Official API: `https://sdmx.oecd.org/public/rest`
Documentation: https://data.oecd.org/api/sdmx-json-documentation/

**What we have:** Data, DataAll, Dataflow, DataflowList, Datastructure, Codelist, ConceptScheme, Availability — 8 variants

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Data | `GET /data/{dataflow_id}/{key}` | YES | `Data` |
| Data | `GET /data/{dataflow_id}/all` | YES | `DataAll` |
| Structure | `GET /dataflow/{agency}/{id}` | YES | `Dataflow` |
| Structure | `GET /dataflow/{agency}` | YES | `DataflowList` |
| Structure | `GET /datastructure/{agency}/{id}` | YES | `Datastructure` |
| Structure | `GET /codelist/{agency}/{id}` | YES | `Codelist` |
| Structure | `GET /conceptscheme/{agency}/{id}` | YES | `ConceptScheme` |
| Availability | `GET /availableconstraint/{flow}/{key}` | YES | `Availability` |
| Structure | `GET /categoryscheme/{agency}` | **NO** | Category scheme (topic groupings) |
| Structure | `GET /categorisation/{agency}` | **NO** | Dataflow-to-category map |
| Data | `GET /data/{flow}/all?updatedAfter={ts}` | **NO** | Delta/incremental fetch |

**Gap count: 3 minor structural endpoints.** Core data access is complete.

---

### 12. World Bank — `intelligence_feeds/economic/worldbank/endpoints.rs`

Official API: `https://api.worldbank.org/v2`
Documentation: https://datahelpdesk.worldbank.org/knowledgebase/articles/889392

**What we have:** IndicatorData, Indicator, IndicatorSearch, Indicators, TopicIndicators, MultiIndicatorData, Country, Countries, IncomeCountries, LendingCountries, Topic, Topics, Source, Sources, IncomeLevels, LendingTypes — 16 endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Indicator | `/country/{code}/indicator/{id}` | YES | `IndicatorData` |
| Indicator | `/indicator/{id}` | YES | `Indicator` |
| Indicator | `/indicator` (search) | YES | `IndicatorSearch` / `Indicators` |
| Indicator | `/topic/{id}/indicator` | YES | `TopicIndicators` |
| Country | `/country/{code}` | YES | `Country` |
| Country | `/country` | YES | `Countries` |
| Country | `/incomelevel/{level}/country` | YES | `IncomeCountries` |
| Country | `/lendingtype/{type}/country` | YES | `LendingCountries` |
| Classification | `/topic/{id}` + `/topic` | YES | `Topic`, `Topics` |
| Classification | `/source/{id}` + `/source` | YES | `Source`, `Sources` |
| Classification | `/incomelevel` + `/lendingtype` | YES | `IncomeLevels`, `LendingTypes` |
| Aggregate | `/region/{code}` | **NO** | Region metadata (EAS, LCN, etc.) |
| Aggregate | `/region` | **NO** | List all regions |
| Aggregate | `/region/{code}/country` | **NO** | Countries in a region |
| Catalog | `/source/{id}/indicator` | **NO** | Indicators for a specific data source |
| Catalog | `/datacatalog` | **NO** | Open Data Catalog (separate service) |

**Gap count: 5 endpoints.** The Region endpoints are important for aggregated regional data (World Bank group regions like East Asia & Pacific, Latin America).

---

## Batch 16 — US Government

### 13. BEA — `intelligence_feeds/us_gov/bea/endpoints.rs`

Official API: `https://apps.bea.gov/api/data`
Documentation: https://apps.bea.gov/api/_pdf/bea_web_service_api_user_guide.pdf

**What we have:** GetDatasetList, GetParameterList, GetParameterValues, GetParameterValuesFiltered, GetData — 5 method variants (all use single base URL with `method=` parameter)

| Category | Endpoint/Method | We Have? | Notes |
|----------|----------------|----------|-------|
| Metadata | `method=GETDATASETLIST` | YES | `GetDatasetList` |
| Metadata | `method=GetParameterList` | YES | `GetParameterList` |
| Metadata | `method=GetParameterValues` | YES | `GetParameterValues` |
| Metadata | `method=GetParameterValuesFiltered` | YES | `GetParameterValuesFiltered` |
| Data | `method=GetData` | YES | `GetData` |

**Gap count: 0.** The BEA API uses a single endpoint URL with method dispatch. All 5 documented methods are implemented. Dataset coverage (NIPA, MNE, ITA, IIP, GDPbyIndustry, Regional, etc.) is also represented in the `BeaDataset` enum.

---

### 14. BLS — `intelligence_feeds/us_gov/bls/endpoints.rs`

Official API: `https://api.bls.gov/publicAPI/v2`
Documentation: https://www.bls.gov/developers/api_signature_v2.htm

**What we have:** TimeSeriesData (`/timeseries/data/`), LatestNumbers (`/timeseries/data/`) — 2 endpoints, **both map to the same path** (the distinction is GET vs POST)

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Data | `POST /timeseries/data/` (multiple series, date range) | YES | `TimeSeriesData` |
| Data | `GET /timeseries/data/{series_id}` (single series) | YES | `LatestNumbers` (same path) |
| Discovery | `GET /surveys` | **NO** | List all BLS surveys with metadata |
| Discovery | `GET /surveys/{survey_prefix}` | **NO** | Metadata for a specific survey |
| Data | `GET /timeseries/popular` | **NO** | Most frequently requested series list |
| Data | `POST /timeseries/data/` with `catalog=true` | **NO** | Series with catalog/description metadata |

**Gap count: 4 discovery/metadata endpoints.** The surveys endpoint is important for discovering BLS series IDs programmatically. The `catalog` query parameter is a feature of `TimeSeriesData`, not a separate endpoint.

---

### 15. Census — `intelligence_feeds/us_gov/census/endpoints.rs`

Official API: `https://api.census.gov/data`
Documentation: https://www.census.gov/data/developers/data-sets.html

**What we have:** Dataset (generic year/dataset query), EconomicIndicator (EITS timeseries), ListDatasets, ListDatasetsAll — 4 variants

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Data | `GET /data/{year}/{dataset}` | YES | `Dataset` |
| Data | `GET /data/timeseries/eits/{indicator}` | YES | `EconomicIndicator` |
| Discovery | `GET /data/{year}.json` | YES | `ListDatasets` |
| Discovery | `GET /data.json` | YES | `ListDatasetsAll` |
| Data | `GET /data/timeseries/poverty/saipe` | **NO** | Small Area Income & Poverty Estimates |
| Data | `GET /data/timeseries/poverty/saipe/schdist` | **NO** | School district poverty estimates |
| Data | `GET /data/timeseries/idb/5year` | **NO** | International Data Base (population) |
| Data | `GET /data/{year}/acs/acs1` | **NO** | American Community Survey 1-year |
| Data | `GET /data/{year}/acs/acs5` | **NO** | American Community Survey 5-year |
| Data | `GET /data/{year}/cbp` | **NO** | County Business Patterns |
| Data | `GET /data/{year}/zbp` | **NO** | ZIP Business Patterns |
| Data | `GET /data/{year}/dec/sf1` | **NO** | Decennial Census |
| Data | `GET /data/{year}/pep/population` | **NO** | Population Estimates Program |

**Gap count: 9 dataset-specific paths.** The generic `Dataset` endpoint covers all of these by substituting year/dataset — so these are not truly missing endpoints but rather missing preset route constants. The SAIPE timeseries is a distinct path pattern that warrants explicit coverage.

---

### 16. Congress — `intelligence_feeds/us_gov/congress/endpoints.rs`

Official API: `https://api.congress.gov/v3`
Documentation: https://github.com/LibraryOfCongress/api.congress.gov

**What we have:** Bills (11 variants), Members (4), Committees (6), Nominations (5), Treaties (4), Congresses (2), Summaries — 33 endpoint variants total

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Bills (11) | `/bill`, `/bill/{congress}/{type}/{number}`, + subresources | YES | All Bill variants |
| Members (4) | `/member`, `/member/{id}`, + legislation | YES | All Member variants |
| Committees (6) | `/committee`, + subresources | YES | All Committee variants |
| Nominations (5) | `/nomination`, + subresources | YES | All Nomination variants |
| Treaties (4) | `/treaty`, + subresources | YES | All Treaty variants |
| Congresses (2) | `/congress`, `/congress/{number}` | YES | Both variants |
| Summaries | `/summaries` | YES | Present |
| Hearings | `GET /hearing` | **NO** | Congressional hearing records |
| Hearings | `GET /hearing/{congress}/{chamber}/{jacketNumber}` | **NO** | Specific hearing details |
| Congressional Record | `GET /congressional-record` | **NO** | Daily congressional record |
| House Communications | `GET /house-communication` | **NO** | House communications |
| Senate Communications | `GET /senate-communication` | **NO** | Senate communications |
| Committee Reports | `GET /committee-report` | **NO** | Committee reports (separate from CommitteeReports sub-endpoint) |
| Committee Prints | `GET /committee-print` | **NO** | Committee prints (top-level, separate) |
| Amendments | `GET /amendment` | **NO** | Top-level amendment list |

**Gap count: 9 endpoints.** Hearings, Congressional Record, and Communications are documented in the API but not implemented. These are valuable for tracking legislative activity.

---

### 17. EIA — `intelligence_feeds/us_gov/eia/endpoints.rs`

Official API: `https://api.eia.gov/v2`
Documentation: https://www.eia.gov/opendata/documentation.php

**What we have:** SeriesData, RouteMetadata, Facets (all parameterized by route string) — 3 generic endpoint variants with 14 route constants

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Data | `GET /v2/{route}/data/` | YES | `SeriesData` |
| Metadata | `GET /v2/{route}/` | YES | `RouteMetadata` |
| Discovery | `GET /v2/{route}/facets/` | YES | `Facets` |
| Routes | petroleum, natural-gas, electricity, coal, total-energy | YES | Route constants present |
| Routes | steo, aeo, international, seds, co2-emissions, densified-biomass | YES | Route constants present |
| Routes | `nuclear-outages` | **NO** | Route constant missing — nuclear plant outage data |
| Routes | `crude-oil-imports` | **NO** | Route constant missing — EIA-814 crude oil imports |
| Routes | `ieo` | **NO** | Route constant missing — International Energy Outlook |
| Routes | `electricity/rto` | **NO** | Route constant missing — Real-time grid operations |
| Routes | `electricity/facility-fuel` | **NO** | Route constant missing — Generation by facility/fuel |
| Routes | `electricity/electric-power-operational-data` | **NO** | Power plant operational data |

**Gap count: 6 missing route constants** (the generic endpoint mechanism is correct). EIA v2 has 14 top-level routes; we cover 8 with explicit constants.

---

## Batch 17 — US Government (continued)

### 18. FBI Crime — `intelligence_feeds/us_gov/fbi_crime/endpoints.rs`

Official API: `https://api.usa.gov/crime/fbi/sapi`
Documentation: https://crime-data-explorer.fr.cloud.gov/api

**What we have:** NationalEstimates, StateEstimates, SummarizedOffense, NationalParticipation, Agencies, NibrsOffender, NibrsVictim — 7 endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Estimates | `GET /api/estimates/national` | YES | `NationalEstimates` |
| Estimates | `GET /api/estimates/states/{state}` | YES | `StateEstimates` |
| Summarized | `GET /api/summarized/state/{state}/{offense}` | YES | `SummarizedOffense` |
| Participation | `GET /api/participation/national` | YES | `NationalParticipation` |
| Agencies | `GET /api/agencies` | YES | `Agencies` |
| NIBRS | `GET /api/nibrs/{offense}/offender/states/{state}/count` | YES | `NibrsOffender` |
| NIBRS | `GET /api/nibrs/{offense}/victim/states/{state}/count` | YES | `NibrsVictim` |
| Arrests | `GET /api/arrest/national/{offense}` | **NO** | National arrest data by offense |
| Arrests | `GET /api/arrest/states/{state}/{offense}` | **NO** | State arrest data by offense |
| Hate Crime | `GET /api/hate-crime/national` | **NO** | National hate crime statistics |
| Hate Crime | `GET /api/hate-crime/states/{state}` | **NO** | State hate crime statistics |
| NIBRS | `GET /api/nibrs/{offense}/weapon/states/{state}/count` | **NO** | Weapon data by offense and state |
| NIBRS | `GET /api/nibrs/{offense}/location/states/{state}/count` | **NO** | Location data by offense and state |
| Participation | `GET /api/participation/states/{state}` | **NO** | State-level participation rates |

**Gap count: 8 endpoints.** Arrest data and hate crime statistics are documented but not implemented. These significantly extend crime data coverage.

---

### 19. SAM.gov — `intelligence_feeds/us_gov/sam_gov/endpoints.rs`

Official API: `https://api.sam.gov`
Documentation: https://open.gsa.gov/api/entity-api/

**What we have:** Entities (`/entity-information/v3/entities`), Opportunities (`/opportunities/v2/search`) — 2 endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Entity | `GET /entity-information/v3/entities` | YES | `Entities` |
| Opportunities | `GET /opportunities/v2/search` | YES | `Opportunities` |
| Exclusions | `GET /entity-information/v4/exclusions` | **NO** | Debarred/excluded parties list |
| Entity | `GET /entity-information/v3/entities/{ueiSAM}` | **NO** | Single entity by UEI |
| Opportunities | `GET /opportunities/v2/search/{opportunityId}` | **NO** | Single opportunity by ID |
| Opportunities | `GET /opportunities/v2/search/{opportunityId}/resources` | **NO** | Opportunity attachments |
| Federal Hierarchy | `GET /federalorganizations/v1/organizations` | **NO** | Federal org hierarchy |
| Extracts | `GET /data-services/v1/extracts` | **NO** | Bulk data extract downloads |
| Assistance | `GET /assistance/v1/search` | **NO** | Assistance listings (grants) |

**Gap count: 8 endpoints.** The Exclusions endpoint (v4) is important for counterparty risk screening. Federal Hierarchy and bulk Extracts are also relevant.

---

### 20. SEC EDGAR — `intelligence_feeds/us_gov/sec_edgar/endpoints.rs`

Official API: `https://data.sec.gov`
Documentation: https://www.sec.gov/edgar/sec-api-documentation

**What we have:** CompanyFilings, CompanyFacts, CompanyConcept, SearchFilings, CompanyTickers, MutualFundTickers, XbrlFrames — 7 endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Company | `GET /submissions/CIK{cik}.json` | YES | `CompanyFilings` |
| XBRL | `GET /api/xbrl/companyfacts/CIK{cik}.json` | YES | `CompanyFacts` |
| XBRL | `GET /api/xbrl/companyconcept/CIK{cik}/{taxonomy}/{tag}.json` | YES | `CompanyConcept` |
| Search | `GET https://efts.sec.gov/LATEST/search-index` | YES | `SearchFilings` |
| Bulk | `GET /files/company_tickers.json` | YES | `CompanyTickers` |
| Bulk | `GET /files/company_tickers_mf.json` | YES | `MutualFundTickers` |
| XBRL Frames | `GET /api/xbrl/frames/{taxonomy}/{tag}/{unit}/CY{period}.json` | YES | `XbrlFrames` |
| Bulk | `GET /files/submissions.zip` | **NO** | Bulk submissions for all filers |
| Bulk | `GET /files/companyfacts.zip` | **NO** | Bulk XBRL facts for all filers |
| Bulk | `GET /files/company_tickers_exchange.json` | **NO** | Tickers with exchange info |
| Full-text | `GET https://efts.sec.gov/LATEST/search-index?q=&dateRange=custom` | **NO** | Full-text search (richer params) |
| Forms | `GET https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany` | **NO** | EDGAR company filing browser (legacy) |

**Gap count: 5 endpoints.** The bulk zip files (`submissions.zip`, `companyfacts.zip`) are the most important gaps — they enable offline analysis of all public company filings without per-CIK calls.

---

### 21. USAspending — `intelligence_feeds/us_gov/usaspending/endpoints.rs`

Official API: `https://api.usaspending.gov/api/v2`
Documentation: https://api.usaspending.gov/

**What we have:** SpendingExplorer, AwardSearch, StateSpending, StateSpecificSpending, Agencies, Glossary, BulkDownloadAwards, FederalAccountAwardCounts, RecipientDuns — 9 endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Spending | `POST /spending/explorer/` | YES | `SpendingExplorer` |
| Search | `POST /search/spending_by_award/` | YES | `AwardSearch` |
| Geography | `GET /spending/state/` | YES | `StateSpending` |
| Geography | `GET /spending/state/{fips}/` | YES | `StateSpecificSpending` |
| Reference | `GET /references/agency/` | YES | `Agencies` |
| Reference | `GET /references/glossary/` | YES | `Glossary` |
| Download | `POST /bulk_download/awards/` | YES | `BulkDownloadAwards` |
| Awards | `GET /awards/count/federal_account/` | YES | `FederalAccountAwardCounts` |
| Recipients | `GET /recipient/duns/` | YES | `RecipientDuns` |
| Awards | `GET /awards/{award_id}/` | **NO** | Single award details |
| Search | `POST /search/spending_by_category/` | **NO** | Spending by recipient category |
| Search | `POST /search/spending_by_geography/` | **NO** | Spending broken down by geography |
| Search | `POST /search/spending_by_transaction/` | **NO** | Transaction-level spending search |
| Search | `POST /search/count/` | **NO** | Count of matching awards |
| Transactions | `GET /transactions/` | **NO** | Transaction history for award |
| Subawards | `POST /subawards/` | **NO** | Sub-award search |
| Disaster | `POST /disaster/federal_account/spending/` | **NO** | COVID/disaster spending by federal account |
| Budget | `GET /federal_accounts/` | **NO** | Federal account list |
| Budget | `GET /financial_balances/agencies/` | **NO** | Agency financial balances |

**Gap count: 11 endpoints.** The spending-by-category and spending-by-geography search endpoints are heavily used for macro federal expenditure analysis. Disaster/COVID spending endpoints are a notable gap.

---

## Batch 18 — Financial

### 22. Alpha Vantage (intel) — `intelligence_feeds/financial/alpha_vantage/endpoints.rs`

Official API: `https://www.alphavantage.co/query`
Documentation: https://www.alphavantage.co/documentation/

**What we have:** GlobalQuote, TimeSeriesIntraday, TimeSeriesDaily, TimeSeriesWeekly, TimeSeriesMonthly, SymbolSearch, CurrencyExchangeRate, FxDaily, CryptoRating, DigitalCurrencyDaily, RealGdp, RealGdpPerCapita, TreasuryYield, FederalFundsRate, Cpi, Inflation, RetailSales, Unemployment, NonfarmPayroll, Sma, Ema, Rsi, Macd, Wti, Brent, NaturalGas, Copper — 27 functions

| Category | Function | We Have? | Notes |
|----------|----------|----------|-------|
| Stocks (6) | GLOBAL_QUOTE, TIME_SERIES_* (4), SYMBOL_SEARCH | YES | All 6 present |
| Stocks | `TIME_SERIES_DAILY_ADJUSTED` | **NO** | Adjusted close for splits/dividends |
| Stocks | `TIME_SERIES_WEEKLY_ADJUSTED` | **NO** | Adjusted weekly |
| Stocks | `TIME_SERIES_MONTHLY_ADJUSTED` | **NO** | Adjusted monthly |
| Stocks | `REALTIME_BULK_QUOTES` | **NO** | Bulk real-time quotes (premium) |
| Stocks | `MARKET_STATUS` | **NO** | Global market open/close status |
| Options | `REALTIME_OPTIONS` | **NO** | Real-time options data (premium) |
| Options | `HISTORICAL_OPTIONS` | **NO** | Historical options chains |
| Intelligence | `NEWS_SENTIMENT` | **NO** | News + sentiment scores (AI-powered) |
| Intelligence | `EARNINGS_CALL_TRANSCRIPT` | **NO** | Earnings call transcripts |
| Intelligence | `TOP_GAINERS_LOSERS` | **NO** | Market movers |
| Intelligence | `INSIDER_TRANSACTIONS` | **NO** | Insider buying/selling |
| Intelligence | `INSTITUTIONAL_HOLDINGS` | **NO** | 13F institutional holdings |
| Fundamentals | `COMPANY_OVERVIEW` | **NO** | Company description + key metrics |
| Fundamentals | `ETF_PROFILE` | **NO** | ETF holdings and metadata |
| Fundamentals | `DIVIDEND` | **NO** | Dividend history |
| Fundamentals | `SPLITS` | **NO** | Stock split history |
| Fundamentals | `INCOME_STATEMENT` | **NO** | Annual/quarterly income statement |
| Fundamentals | `BALANCE_SHEET` | **NO** | Annual/quarterly balance sheet |
| Fundamentals | `CASH_FLOW` | **NO** | Annual/quarterly cash flow |
| Fundamentals | `EARNINGS` | **NO** | EPS history (actual vs estimate) |
| Fundamentals | `EARNINGS_ESTIMATES` | **NO** | Forward EPS estimates |
| Fundamentals | `EARNINGS_CALENDAR` | **NO** | Upcoming earnings schedule |
| Fundamentals | `IPO_CALENDAR` | **NO** | Upcoming IPO schedule |
| Crypto | `DIGITAL_CURRENCY_WEEKLY` | **NO** | Weekly crypto OHLCV |
| Crypto | `DIGITAL_CURRENCY_MONTHLY` | **NO** | Monthly crypto OHLCV |
| Forex | `FX_INTRADAY` | **NO** | Intraday forex OHLCV |
| Forex | `FX_WEEKLY` | **NO** | Weekly forex OHLCV |
| Forex | `FX_MONTHLY` | **NO** | Monthly forex OHLCV |
| Commodities | `ALUMINUM` | **NO** | Aluminum prices |
| Commodities | `WHEAT` | **NO** | Wheat prices |
| Commodities | `CORN` | **NO** | Corn prices |
| Commodities | `COTTON` | **NO** | Cotton prices |
| Commodities | `SUGAR` | **NO** | Sugar prices |
| Commodities | `COFFEE` | **NO** | Coffee prices |
| Commodities | `ALL_COMMODITIES` | **NO** | Aggregate commodity index |
| Technical | BBANDS, STOCH, ADX, ATR, OBV, AD, ADOSC, + 40 more | **NO** | 50+ technical indicators beyond the 4 implemented |
| Note | `CRYPTO_RATING` | Present but DEPRECATED | No longer in official docs |

**Gap count: 37+ functions.** The highest-priority gaps are: `NEWS_SENTIMENT` (AI-powered news analysis), `COMPANY_OVERVIEW`, `EARNINGS`, `INCOME_STATEMENT`, `BALANCE_SHEET`, `EARNINGS_CALENDAR`. These are the most commonly used fundamentals functions.

---

### 23. Finnhub (intel) — `intelligence_feeds/financial/finnhub/endpoints.rs`

Official API: `https://finnhub.io/api/v1`
Documentation: https://finnhub.io/docs/api

**What we have:** Quote, StockCandles, SymbolSearch, CompanyProfile, CompanyPeers, Financials, BasicFinancials, MarketNews, CompanyNews, MarketStatus, EarningsCalendar, IpoCalendar, ForexRates, ForexCandles, ForexSymbols, CryptoCandles, CryptoSymbols, EconomicCalendar, CountryList, EconomicData, SocialSentiment, InsiderTransactions, InsiderSentiment, RecommendationTrends — 24 endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Stock (7) | /quote, /stock/candle, /search, /stock/profile2, /stock/peers, /stock/financials, /stock/metric | YES | All 7 |
| Market (5) | /news, /company-news, /stock/market-status, /calendar/earnings, /calendar/ipo | YES | All 5 |
| Forex (3) | /forex/rates, /forex/candle, /forex/symbol | YES | All 3 |
| Crypto (2) | /crypto/candle, /crypto/symbol | YES | Both |
| Economic (3) | /calendar/economic, /country, /economic | YES | All 3 |
| Sentiment (4) | /stock/social-sentiment, /stock/insider-transactions, /stock/insider-sentiment, /stock/recommendation | YES | All 4 |
| Fundamentals | `GET /stock/dividends` | **NO** | Dividend history |
| Fundamentals | `GET /stock/splits` | **NO** | Stock split history |
| Fundamentals | `GET /stock/earnings` | **NO** | Reported EPS vs estimates |
| Fundamentals | `GET /stock/eps-estimate` | **NO** | Forward EPS consensus |
| Fundamentals | `GET /stock/revenue-estimate` | **NO** | Revenue consensus estimates |
| Fundamentals | `GET /stock/esg` | **NO** | ESG scores |
| Fundamentals | `GET /stock/upgrades-downgrades` | **NO** | Analyst rating changes |
| Fundamentals | `GET /stock/price-target` | **NO** | Analyst price targets |
| Index | `GET /index/constituents` | **NO** | Index constituent list |
| Index | `GET /index/historical-constituents` | **NO** | Historical index composition |
| ETF | `GET /etf/profile` | **NO** | ETF fund profile |
| ETF | `GET /etf/holdings` | **NO** | ETF holdings |
| Mutual Fund | `GET /mutual-fund/profile` | **NO** | MF profile |
| Mutual Fund | `GET /mutual-fund/holdings` | **NO** | MF holdings |
| Bond | `GET /bond/candle` | **NO** | Bond price OHLCV |
| Bond | `GET /bond/profile` | **NO** | Bond metadata |
| Ownership | `GET /stock/institutional-ownership` | **NO** | 13F institutional holders |
| Ownership | `GET /stock/fund-ownership` | **NO** | Fund ownership |
| Filings | `GET /stock/filings` | **NO** | SEC filings list |
| Filings | `GET /international/filings` | **NO** | Non-US regulatory filings |
| Alternative | `GET /stock/supply-chain` | **NO** | Supply chain relationships |
| Alternative | `GET /stock/lobbying` | **NO** | Lobbying disclosure |
| Alternative | `GET /stock/usa-spending` | **NO** | Government contract spending |
| Transcripts | `GET /stock/transcripts` | **NO** | Earnings call transcripts list |
| Transcripts | `GET /stock/transcripts/list` | **NO** | Available transcripts |
| Crypto | `GET /crypto/exchange` | **NO** | Crypto exchange list |
| WebSocket | `wss://ws.finnhub.io` (trades, news) | **PARTIAL** | ws_base present but no subscribe logic |

**Gap count: 28 REST endpoints + WebSocket subscription logic.** Highest priority: `/stock/earnings`, `/stock/dividends`, `/stock/upgrades-downgrades`, `/stock/price-target`, ETF and institutional ownership endpoints.

---

### 24. NewsAPI — `intelligence_feeds/financial/newsapi/endpoints.rs`

Official API: `https://newsapi.org/v2`
Documentation: https://newsapi.org/docs/endpoints

**What we have:** TopHeadlines, Everything, Sources — 3 endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Headlines | `GET /v2/top-headlines` | YES | `TopHeadlines` |
| Search | `GET /v2/everything` | YES | `Everything` |
| Discovery | `GET /v2/top-headlines/sources` | YES | `Sources` |

**Gap count: 0.** NewsAPI has exactly 3 endpoints (plus the Sources endpoint which is `/top-headlines/sources`, already implemented correctly). Coverage is complete.

---

### 25. OpenFIGI — `intelligence_feeds/financial/openfigi/endpoints.rs`

Official API: `https://api.openfigi.com`
Documentation: https://www.openfigi.com/api/documentation

**What we have:** Mapping (`/v3/mapping`), Search (`/v3/search`), MappingValues (`/v3/mapping/values/{key}`) — 3 endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Core | `POST /v3/mapping` | YES | `Mapping` |
| Search | `POST /v3/search` | YES | `Search` |
| Discovery | `GET /v3/mapping/values/{key}` | YES | `MappingValues` |
| Filter | `POST /v3/filter` | **NO** | Filter FIGIs by keywords and optional field filters |

**Gap count: 1 endpoint.** The `/v3/filter` endpoint was added in v3 alongside mapping and search and is documented but not implemented. It allows narrowing results by exchange, market sector, security type, etc.

---

## Batch 19 — Crypto

### 26. CoinGecko — `intelligence_feeds/crypto/coingecko/endpoints.rs`

Official API: `https://api.coingecko.com/api/v3`
Documentation: https://docs.coingecko.com/reference/introduction

**What we have:** SimplePrice, CoinsList, CoinDetail, CoinMarketChart, CoinsMarkets, CoinTickers, Search, SearchTrending, Global, GlobalDefi, Exchanges, ExchangeDetail — 12 endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Simple | `GET /simple/price` | YES | `SimplePrice` |
| Coins | `GET /coins/list` | YES | `CoinsList` |
| Coins | `GET /coins/{id}` | YES | `CoinDetail` |
| Coins | `GET /coins/{id}/market_chart` | YES | `CoinMarketChart` |
| Coins | `GET /coins/markets` | YES | `CoinsMarkets` |
| Coins | `GET /coins/{id}/tickers` | YES | `CoinTickers` |
| Search | `GET /search` | YES | `Search` |
| Search | `GET /search/trending` | YES | `SearchTrending` |
| Global | `GET /global` | YES | `Global` |
| Global | `GET /global/decentralized_finance_defi` | YES | `GlobalDefi` |
| Exchanges | `GET /exchanges` | YES | `Exchanges` |
| Exchanges | `GET /exchanges/{id}` | YES | `ExchangeDetail` |
| Simple | `GET /simple/token_price/{platform_id}` | **NO** | Token price by contract address |
| Simple | `GET /simple/supported_vs_currencies` | **NO** | List of supported vs-currencies |
| Coins | `GET /coins/{id}/ohlc` | **NO** | OHLC candlestick data |
| Coins | `GET /coins/{id}/market_chart/range` | **NO** | Market chart with Unix timestamp range |
| Coins | `GET /coins/{id}/history` | **NO** | Historical snapshot on a specific date |
| Coins | `GET /coins/{id}/contract/{contract_address}` | **NO** | Coin data by token contract address |
| Categories | `GET /coins/categories/list` | **NO** | List all coin categories |
| Categories | `GET /coins/categories` | **NO** | Coins categorized with market data |
| NFTs | `GET /nfts/list` | **NO** | NFT collection list |
| NFTs | `GET /nfts/{id}` | **NO** | NFT collection detail |
| NFTs | `GET /nfts/{asset_platform_id}/contract/{contract_address}` | **NO** | NFT by contract |
| Exchanges | `GET /exchanges/{id}/tickers` | **NO** | Exchange tickers |
| Exchanges | `GET /exchanges/{id}/volume_chart` | **NO** | Exchange volume history |
| Derivatives | `GET /derivatives` | **NO** | Derivatives market overview |
| Derivatives | `GET /derivatives/exchanges` | **NO** | Derivatives exchanges list |
| Asset Platforms | `GET /asset_platforms` | **NO** | Blockchain platforms list |
| Companies | `GET /companies/public_treasury/{coin_id}` | **NO** | Public company BTC/ETH treasury holdings |

**Gap count: 18 endpoints.** Highest priority gaps: `/coins/{id}/ohlc` (OHLCV data), `/coins/{id}/market_chart/range` (precise date-range charts), `/simple/supported_vs_currencies`, `/coins/categories`, and the token contract address endpoints.

---

### 27. Coinglass — `intelligence_feeds/crypto/coinglass/endpoints.rs`

Official API: `https://open-api-v4.coinglass.com`
Documentation: https://docs.coinglass.com/reference/endpoint-overview

**What we have:** 48 endpoints across all major categories

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Market Discovery (4) | SupportedCoins, SupportedExchangePairs, PairsMarkets, CoinsMarkets | YES | All 4 |
| Liquidations (4) | History, Heatmap, Map, MaxPain | YES | All 4 |
| Open Interest (5) | Ohlc, Aggregated, History, VolRatio, ByCoin | YES | All 5 |
| Funding Rates (3) | History, Current, Aggregated | YES | All 3 |
| Long/Short (6) | RateHistory, AccountRatio, GlobalAccountRatio, TopLongShortPosition, TopLongShortAccount, TakerBuySell | YES | All 6 |
| Order Book (3) | BidAskRange, OrderbookHeatmap, LargeOrders | YES | All 3 |
| Volume & Flows (3) | CVD, NetFlow, Footprint | YES | All 3 |
| Options (3) | MaxPain, OiHistory, VolumeHistory | YES | All 3 |
| On-Chain (6) | ExchangeReserve, BalanceHistory, Erc20Transfers, WhaleTransfers, TokenUnlocks, TokenVesting | YES | All 6 |
| ETF (6) | BtcFlow, EthFlow, SolFlow, XrpFlow, HkFlow, GrayscalePremium | YES | All 6 |
| Hyperliquid (4) | WhaleAlert, WhalePositions, WalletPositions, PositionDistribution | YES | All 4 |
| Technical (2) | RSI, MovingAverage | YES | Both |
| Spot | `GET /api/spot/supported-coins` | **NO** | Spot market supported coins |
| Spot | `GET /api/spot/pairs-markets` | **NO** | Spot market pairs data |
| Spot | `GET /api/spot/orderbook/heatmap` | **NO** | Spot order book heatmap |
| Spot | `GET /api/spot/taker-buy-sell-volume/chart` | **NO** | Spot taker buy/sell |
| Options | `GET /api/options/info` | **NO** | Options info (BTC/ETH open interest by expiry) |
| Indicators | `GET /api/indicator/ema` | **NO** | EMA indicator |
| Indicators | `GET /api/indicator/macd` | **NO** | MACD indicator |
| Indicators | `GET /api/indicator/boll` | **NO** | Bollinger Bands |
| Indicators | Various index indicators (fear & greed, etc.) | **NO** | Multiple index indicators |

**Gap count: 9+ endpoints.** The Spot category endpoints (spot market equivalents of the futures endpoints) are completely missing. Options info and additional technical indicators (EMA, MACD, BOLL) are also absent.

---

## Summary Table

| # | Provider | File | Total We Have | Key Gaps | Priority |
|---|----------|------|--------------|----------|----------|
| 1 | BIS | `economic/bis` | 8 | 4 minor structural | Low |
| 2 | BOE | `economic/boe` | 2 | 2 (no real API for discovery) | None |
| 3 | Bundesbank | `economic/bundesbank` | 8 | 3 minor structural | Low |
| 4 | CBR | `economic/cbr` | 10 | 6 (RUONIA, deposit/refinancing rates) | Medium |
| 5 | ECB | `economic/ecb` | 6 | 4 structural | Low |
| 6 | DBnomics | `economic/dbnomics` | 10 | 2 (param variants) | None |
| 7 | ECOS | `economic/ecos` | 6 | **0** | Complete |
| 8 | Eurostat | `economic/eurostat` | 9 | 4 (DCAT-AP, RSS, metabase) | Low |
| 9 | FRED | `economic/fred` | 32 | **0** | Complete |
| 10 | IMF | `economic/imf` | 5 | 5 (DataMapper API missing entirely) | **High** |
| 11 | OECD | `economic/oecd` | 8 | 3 minor structural | Low |
| 12 | World Bank | `economic/worldbank` | 16 | 5 (Region endpoints) | Medium |
| 13 | BEA | `us_gov/bea` | 5 | **0** | Complete |
| 14 | BLS | `us_gov/bls` | 2 | 4 (surveys, popular series) | Medium |
| 15 | Census | `us_gov/census` | 4 | 9 (mostly route constants) | Medium |
| 16 | Congress | `us_gov/congress` | 33 | 9 (hearings, record, communications) | Medium |
| 17 | EIA | `us_gov/eia` | 3+14 routes | 6 missing route constants | Medium |
| 18 | FBI Crime | `us_gov/fbi_crime` | 7 | 8 (arrests, hate crime, NIBRS variants) | Medium |
| 19 | SAM.gov | `us_gov/sam_gov` | 2 | 8 (exclusions, federal hierarchy) | **High** |
| 20 | SEC EDGAR | `us_gov/sec_edgar` | 7 | 5 (bulk zips, tickers with exchange) | **High** |
| 21 | USAspending | `us_gov/usaspending` | 9 | 11 (search variants, disaster, budget) | Medium |
| 22 | Alpha Vantage | `financial/alpha_vantage` | 27 | 37+ (fundamentals, news, adjusted) | **High** |
| 23 | Finnhub | `financial/finnhub` | 24 | 28 (earnings, dividends, ETF, ownership) | **High** |
| 24 | NewsAPI | `financial/newsapi` | 3 | **0** | Complete |
| 25 | OpenFIGI | `financial/openfigi` | 3 | 1 (`/v3/filter`) | Low |
| 26 | CoinGecko | `crypto/coingecko` | 12 | 18 (OHLC, range chart, categories, NFT) | **High** |
| 27 | Coinglass | `crypto/coinglass` | 48 | 9+ (spot category, EMA/MACD/BOLL) | Medium |

---

## Top Priority Gaps

### P0 — Immediate Value

1. **Alpha Vantage** — `NEWS_SENTIMENT`, `COMPANY_OVERVIEW`, `INCOME_STATEMENT`, `BALANCE_SHEET`, `EARNINGS`, `EARNINGS_CALENDAR`, `DIVIDEND`, `SPLITS` (8 high-use fundamentals/intelligence functions completely absent)
2. **Finnhub** — `/stock/earnings`, `/stock/dividends`, `/stock/upgrades-downgrades`, `/stock/price-target`, `/etf/holdings`, `/stock/institutional-ownership` (6 core fundamentals missing)
3. **CoinGecko** — `/coins/{id}/ohlc`, `/coins/{id}/market_chart/range`, `/coins/categories`, `/simple/supported_vs_currencies`, `/simple/token_price/{platform_id}` (OHLCV + DeFi token data)
4. **IMF DataMapper API** — entirely separate API for WEO forecasts not implemented at all (5 endpoints)
5. **SEC EDGAR** — bulk `submissions.zip` and `companyfacts.zip` for offline/batch analysis

### P1 — Important Extensions

6. **SAM.gov** — Exclusions API v4 (debarred parties), Federal Hierarchy
7. **Congress** — Hearings, Congressional Record, Senate/House Communications
8. **BLS** — `GET /surveys` for programmatic series discovery
9. **World Bank** — Region endpoints (`/region`, `/region/{code}/country`)
10. **Coinglass** — Spot market category (currently only futures covered)
11. **USAspending** — spending-by-category/geography search, disaster spending
12. **CBR** — RUONIA rate, refinancing rate, deposit rates
13. **FBI Crime** — Arrest data, hate crime statistics

### P2 — Nice to Have

14. **EIA** — Missing route constants: `nuclear-outages`, `crude-oil-imports`, `ieo`, `electricity/rto`
15. **Eurostat** — Metabase bulk file, DCAT-AP
16. **OpenFIGI** — `/v3/filter` endpoint
17. **SDMX structural** (BIS, Bundesbank, ECB, OECD) — categoryscheme, categorisation, contentconstraint

---

## Sources

- [BIS Data Portal API Documentation](https://data.bis.org/help/tools)
- [BIS SDMX REST API v2 Spec](https://stats.bis.org/api-doc/v2/)
- [Bank of England Database Help](https://www.bankofengland.co.uk/boeapps/database/help.asp)
- [Bundesbank SDMX Web Service](https://www.bundesbank.de/en/statistics/time-series-databases/help-for-sdmx-web-service)
- [CBR API (unofficial reference)](https://modern-cms.ru/dev/)
- [ECB Data Portal SDMX API](https://data.ecb.europa.eu/help/api/overview)
- [DBnomics Web API Documentation](https://docs.db.nomics.world/web-api/)
- [ECOS Bank of Korea API](https://ecos.bok.or.kr/api/)
- [Eurostat API Introduction](https://ec.europa.eu/eurostat/web/user-guides/data-browser/api-data-access/api-introduction)
- [Eurostat Catalogue API](https://ec.europa.eu/eurostat/web/user-guides/data-browser/api-data-access/api-getting-started/catalogue-api)
- [FRED API Documentation](https://fred.stlouisfed.org/docs/api/fred/)
- [IMF JSON RESTful Data Services](http://datahelp.imf.org/knowledgebase/articles/667681)
- [IMF DataMapper API Help](https://www.imf.org/external/datamapper/api/help)
- [OECD SDMX-JSON API Documentation](https://data.oecd.org/api/sdmx-json-documentation/)
- [World Bank Indicators API](https://datahelpdesk.worldbank.org/knowledgebase/articles/898599-indicator-api-queries)
- [BEA API User Guide (PDF, Nov 2024)](https://apps.bea.gov/api/_pdf/bea_web_service_api_user_guide.pdf)
- [BLS API v2 Signatures](https://www.bls.gov/developers/api_signature_v2.htm)
- [Census Available APIs](https://www.census.gov/data/developers/data-sets.html)
- [Congress.gov API GitHub](https://github.com/LibraryOfCongress/api.congress.gov)
- [EIA OpenData Documentation](https://www.eia.gov/opendata/documentation.php)
- [FBI Crime Data Explorer API](https://cde.ucr.cjis.gov/)
- [SAM.gov Entity Management API](https://open.gsa.gov/api/entity-api/)
- [SAM.gov Exclusions API v4](https://open.gsa.gov/api/exclusions-api/)
- [SEC EDGAR API Documentation](https://www.sec.gov/search-filings/edgar-application-programming-interfaces)
- [USAspending API](https://api.usaspending.gov/)
- [Alpha Vantage API Documentation](https://www.alphavantage.co/documentation/)
- [Finnhub API Documentation](https://finnhub.io/docs/api)
- [NewsAPI Endpoints Documentation](https://newsapi.org/docs/endpoints)
- [OpenFIGI API Documentation](https://www.openfigi.com/api/documentation)
- [CoinGecko API v3](https://docs.coingecko.com/reference/introduction)
- [Coinglass Endpoint Overview](https://docs.coinglass.com/reference/endpoint-overview)
