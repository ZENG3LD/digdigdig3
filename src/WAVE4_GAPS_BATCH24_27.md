# WAVE4 Endpoint Gap Analysis — Batches 24–27
## Conflict/Humanitarian, Space, Aviation, Maritime

Analyzed: 19 connectors across 4 domains.
Method: read each `endpoints.rs`, cross-reference official API documentation.

---

## Batch 24 — Conflict / Humanitarian

### 1. ACLED — `intelligence_feeds/conflict/acled/endpoints.rs`

**Current implementation:** 1 endpoint (`Events` → `""` on base `https://api.acleddata.com/acled/read`)

Official docs: https://acleddata.com/acled-api-documentation

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Core Events | `GET /acled/read` | YES (as `Events`) | Main endpoint, all filtering via query params |
| CAST Forecasts | `GET /cast/` | NO | Conflict Alert System Tool — predictive conflict forecasting |
| Deleted Records | `GET /deleted/` | NO | Track removed/corrected records, useful for data integrity |

**Summary:** 2 missing endpoints. The `cast/` endpoint provides forward-looking conflict alerts which is high-value for an intelligence feed. The `deleted/` endpoint supports data synchronization.

---

### 2. GDELT — `intelligence_feeds/conflict/gdelt/endpoints.rs`

**Current implementation:** 4 endpoints (DocApi, GeoApi, TvApi, ContextApi)

Official docs: https://blog.gdeltproject.org/gdelt-doc-2-0-api-debuts/

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| DOC API | `GET /doc/doc` | YES | Full-text news search |
| GEO API | `GET /geo/geo` | YES | Geographic event search |
| TV API | `GET /tv/tv` | YES | Television news search |
| Context API | `GET /context/context` | YES | Sentence-level search (72h window) |
| DOC mode: `timelinevolinfo` | (query param) | PARTIAL | Mode exists but not represented as a `DocMode` variant |
| DOC mode: `tonechart` | (query param) | PARTIAL | Mode in TV already, but also valid for DOC — no `TonChart` variant for `DocMode` |

**Summary:** All 4 main endpoints are present. Minor gap: `DocMode` enum is missing `TimelineVolInfo` and `ToneChart` modes that the DOC API supports (these are query-param level, not new endpoints). Coverage is functionally complete.

---

### 3. ReliefWeb — `intelligence_feeds/conflict/reliefweb/endpoints.rs`

**Current implementation:** 6 endpoints (Reports, Disasters, Countries, Jobs, Training, Sources)

Official docs: https://apidoc.reliefweb.int/endpoints

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Reports | `GET /reports` | YES | Main humanitarian content |
| Disasters | `GET /disasters` | YES | Crisis/disaster metadata |
| Countries | `GET /countries` | YES | Country profiles |
| Jobs | `GET /jobs` | YES | Humanitarian job listings |
| Training | `GET /training` | YES | Training opportunities |
| Sources | `GET /sources` | YES | Organizations/partners |
| Blog | `GET /blog` | NO | ReliefWeb editorial blog posts |
| Book | `GET /book` | NO | Static site info, help pages, taxonomy descriptions |
| References | `GET /references` | NO | Taxonomy/facet reference data — editorial tags used across all content |
| API Version | Base uses `/v1` | PARTIAL | Official recommends `/v2` — both work, but v2 is current |

**Summary:** 3 missing endpoints. `References` is useful for tag/filter lookups. `Blog` and `Book` are lower priority but complete the API surface. Base URL should be updated from `/v1` to `/v2`.

---

### 4. UCDP — `intelligence_feeds/conflict/ucdp/endpoints.rs`

**Current implementation:** 5 endpoints (GeoEvents `/gedevents/24.1`, BattleDeaths `/battledeaths/24.1`, NonState `/nonstate/24.1`, OneSided `/onesided/24.1`, StateConflict `/stateconflict/24.1`)

Official docs: https://ucdp.uu.se/apidocs/

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| GeoEvents | `/gedevents/{version}` | YES | Georeferenced event dataset |
| Battle Deaths | `/battledeaths/{version}` | YES | Battle-related deaths |
| Non-State | `/nonstate/{version}` | YES | Non-state conflicts |
| One-Sided | `/onesided/{version}` | YES | One-sided violence |
| State Conflict | `/stateconflict/{version}` | YES | Mapped to `/ucdpprioconflict` in API — name mismatch |
| Dyadic Dataset | `/dyadic/{version}` | NO | UCDP Dyadic conflict dataset — unique actor-pair combinations per year |
| Version hardcoded | `24.1` throughout | STALE | Latest versions are `25.1` for yearly datasets, `26.0.1` for gedevents (candidate) |

**Summary:** 1 missing endpoint (`dyadic`). Version numbers are stale — `24.1` should be `25.1`. The `StateConflict` endpoint path should be `/ucdpprioconflict/{version}` not `/stateconflict/{version}` — the latter does not exist in the official API.

---

### 5. UNHCR — `intelligence_feeds/conflict/unhcr/endpoints.rs`

**Current implementation:** 5 endpoints (Population, Demographics, Solutions, AsylumDecisions, Countries)

Official docs: https://api.unhcr.org/docs/refugee-statistics.html

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Population | `GET /population/` | YES | Refugee and displaced persons stats |
| Demographics | `GET /demographics/` | YES | Age/sex breakdowns |
| Solutions | `GET /solutions/` | YES | Resettlement, returns, naturalization |
| Asylum Decisions | `GET /asylum-decisions/` | YES | RSD decision outcomes |
| Countries | `GET /countries/` | YES | Country reference list |
| Asylum Applications | `GET /asylum-applications/` | NO | Applications submitted (distinct from decisions) — key distinction for tracking asylum flows |
| IDP (Internal Displacement) | Not confirmed at `/idp/` | UNCLEAR | Some UNHCR data includes IDP stats — may be rolled into `population/` params |

**Summary:** 1 confirmed missing endpoint (`asylum-applications/`). The distinction between applications and decisions is significant for tracking asylum processing. The `Demographics` endpoint may overlap with `/population/` using demographic filters — needs verification.

---

## Batch 25 — Space

### 6. Launch Library 2 — `intelligence_feeds/space/launch_library/endpoints.rs`

**Current implementation:** 10 endpoints across launches, events, astronauts, space stations, agencies, rockets, spacecraft.

Official docs: https://ll.thespacedevs.com/docs
Note: API is at v2.3.0 (Sep 2024) — implementation uses v2.2.0.

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Launch Upcoming | `GET /launch/upcoming/` | YES | |
| Launch Previous | `GET /launch/previous/` | YES | |
| Launch Detail | `GET /launch/{id}/` | YES | |
| Event Upcoming | `GET /event/upcoming/` | YES | |
| Event Previous | `GET /event/previous/` | YES | |
| Astronaut | `GET /astronaut/` | YES | |
| Space Station | `GET /space_station/` | YES | Mapped to `/spacestation/` in v2.3.0 — path may be wrong |
| Agency | `GET /agency/` | YES | Mapped to `/agencies/` in v2.3.0 — plural |
| Rocket Config | `GET /config/launcher/` | YES | |
| Spacecraft Config | `GET /config/spacecraft/` | YES | |
| Launcher (reusable stages) | `GET /launcher/` | NO | Tracks reusable rocket stages (e.g., Falcon 9 cores) — serial numbers, flight counts |
| Launch Pad | `GET /pad/` | NO | Specific launch pads (e.g., SLC-40) — more granular than location |
| Location | `GET /location/` | NO | Launch site locations |
| Expedition | `GET /expedition/` | NO | Space station crew expeditions |
| Docking Event | `GET /docking_event/` | NO | Spacecraft docking events (ISS visiting vehicles) |
| Payload | `GET /payload/` | NO | Mission payloads (new in v2.3.0) |
| API Version | Uses `2.2.0` | STALE | Latest is `2.3.0` (Sep 2024) |

**Summary:** 6 missing endpoints. `launcher` (reusable stages), `pad`, `location`, `expedition`, and `docking_event` are all real LL2 resources. API version is stale (2.2.0 vs current 2.3.0). Some path names differ in v2.3.0 (`/agencies/` not `/agency/`, `/spacestation/` not `/space_station/`).

---

### 7. NASA — `intelligence_feeds/space/nasa/endpoints.rs`

**Current implementation:** 9 endpoints (NeoFeed, NeoLookup, DonkiCme, DonkiGst, DonkiFlr, DonkiSep, DonkiIps, Apod, EpicNatural)

Official docs: https://api.nasa.gov

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| APOD | `GET /planetary/apod` | YES | Astronomy Picture of the Day |
| NEO Feed | `GET /neo/rest/v1/feed` | YES | NEO by date range |
| NEO Lookup | `GET /neo/rest/v1/neo/{id}` | YES | Specific asteroid |
| NEO Browse | `GET /neo/rest/v1/neo/browse/` | NO | Browse full NEO catalog (paginated) |
| DONKI CME | `GET /DONKI/CME` | YES | Coronal mass ejections |
| DONKI CME Analysis | `GET /DONKI/CMEAnalysis` | NO | Analyzed CME data with speed/halfAngle filters |
| DONKI GST | `GET /DONKI/GST` | YES | Geomagnetic storms |
| DONKI FLR | `GET /DONKI/FLR` | YES | Solar flares |
| DONKI SEP | `GET /DONKI/SEP` | YES | Solar energetic particles |
| DONKI IPS | `GET /DONKI/IPS` | YES | Interplanetary shocks |
| DONKI Notifications | `GET /DONKI/notifications` | NO | SWRC notification messages — high-value for alerts |
| DONKI MPC | `GET /DONKI/MPC` | NO | Magnetopause crossings |
| DONKI RBE | `GET /DONKI/RBE` | NO | Radiation belt enhancements |
| DONKI HSS | `GET /DONKI/HSS` | NO | High-speed streams |
| DONKI WEP | `GET /DONKI/WEP` | NO | WSA+EnlilSimulation predictions |
| EPIC Natural | `GET /EPIC/api/natural` | YES | Earth full-disc natural color images |
| EPIC Enhanced | `GET /EPIC/api/enhanced` | NO | Enhanced color imagery |
| Mars Rover Photos | `GET /mars-photos/api/v1/rovers/{rover}/photos` | NO | Curiosity/Opportunity/Spirit imagery |
| Earth Imagery | `GET /planetary/earth/imagery` | NO | Landsat satellite imagery by coords |
| Earth Assets | `GET /planetary/earth/assets` | NO | Check if imagery available for coords |
| Insight Weather | `GET /insight_weather/` | NO | Mars InSight lander weather data (may be inactive) |

**Summary:** 11+ missing endpoints. The DONKI suite is significantly incomplete — `CMEAnalysis`, `notifications`, `MPC`, `RBE`, `HSS`, and `WEP` are all missing. Mars Rover Photos and Earth Imagery are entirely absent. `NEO Browse` for full catalog access is missing.

---

### 8. Sentinel Hub — `intelligence_feeds/space/sentinel_hub/endpoints.rs`

**Current implementation:** 4 endpoints (Token, CatalogSearch, Process, Statistical)

Official docs: https://docs.sentinel-hub.com/api/latest/api/overview/

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| OAuth Token | `POST /oauth/token` | YES | Auth token |
| Catalog Search | `POST /api/v1/catalog/search` | YES | STAC search (mapped as POST, not GET) |
| Catalog Collections | `GET /api/v1/catalog/1.0.0/collections` | NO | List all available satellite collections |
| Catalog Collections (STAC) | `GET /api/v1/catalog/1.0.0/` | NO | Entry point, links to all catalog sub-resources |
| Process | `POST /api/v1/process` | YES | Process/render satellite imagery |
| Batch Process | `POST /api/v1/batch/process` | NO | Process large regions/timeframes — delivers to object storage |
| Statistical | `POST /api/v1/statistical` | YES | Statistical analysis without download |
| Batch Statistical | `POST /api/v1/batch/statistics` | NO | Batch statistics for multiple polygons |
| OGC WMS | `GET /ogc/wms/{instance_id}` | NO | OGC Web Map Service — standard GIS interoperability |
| OGC WCS | `GET /ogc/wcs/{instance_id}` | NO | OGC Web Coverage Service |
| OGC WFS | `GET /ogc/wfs/{instance_id}` | NO | OGC Web Feature Service |
| Async Process | `POST /api/v1/async/process` | NO | Beta async processing endpoint |

**Summary:** 7 missing endpoints. Batch Processing and Batch Statistical are important for production-scale use cases. Collections listing endpoint is needed to discover available satellite data sources. OGC endpoints (WMS/WCS/WFS) provide standard GIS interoperability. The current `CatalogSearch` path is also slightly wrong — official path is `/api/v1/catalog/1.0.0/` (not `/api/v1/catalog/search`).

---

### 9. Space-Track — `intelligence_feeds/space/space_track/endpoints.rs`

**Current implementation:** 7 endpoints (Login, SatelliteCatalog, GeneralPerturbations, Decay, Debris, LaunchSites, Tip). All encoded as hardcoded query strings rather than a flexible query builder.

Official docs: https://www.space-track.org/documentation

| Category | Endpoint/Class | We Have? | Notes |
|----------|----------------|----------|-------|
| Auth | `POST /ajaxauth/login` | YES | Session cookie auth |
| Satellite Catalog | `/basicspacedata/query/class/satcat/...` | YES | Via `SatelliteCatalog` |
| General Perturbations (TLE) | `/basicspacedata/query/class/gp/...` | YES | Via `GeneralPerturbations` |
| GP History | `/basicspacedata/query/class/gp_history/...` | NO | Historical TLE/ephemeris data — critical for backtesting orbital predictions |
| Decay | `/basicspacedata/query/class/decay/...` | YES | |
| TIP | `/basicspacedata/query/class/tip/...` | YES | Tracking & Impact Predictions |
| Launch Sites | `/basicspacedata/query/class/launch_site/...` | YES | |
| Boxscore | `/basicspacedata/query/class/boxscore/...` | NO | Satellite count by country — orbital object inventory |
| Conjunction Data Messages | `/basicspacedata/query/class/cdm_public/...` | NO | Public CDM — close approach / collision risk data |
| Satcat Change | `/basicspacedata/query/class/satcat_change/...` | NO | Tracks changes to satellite catalog entries |
| Satcat Debut | `/basicspacedata/query/class/satcat_debut/...` | NO | Newly appeared objects in catalog |
| OMM | `/basicspacedata/query/class/omm/...` | NO | Orbit Mean-Elements Message (modern XML/JSON format for TLEs) |
| Announcement | `/basicspacedata/query/class/announcement/...` | NO | Site announcements and service notifications |
| Query Architecture | Hardcoded strings | POOR | Should be a generic query builder: `/basicspacedata/query/class/{class}/{predicates}` |

**Summary:** 7 missing classes. The most critical are `cdm_public` (conjunction/collision data) and `gp_history` (historical TLEs). The implementation architecture is also problematic — hardcoded query strings instead of a flexible predicate builder means adding new query patterns requires new enum variants.

---

### 10. SpaceX — `intelligence_feeds/space/spacex/endpoints.rs`

**Current implementation:** 12 endpoints (LaunchesAll/Latest/Next/Upcoming/Past, Rockets, Crew, Starlink, Launchpads, Landpads, Payloads, Capsules)

Official docs: https://github.com/r-spacex/SpaceX-API

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Launches (all) | `GET /launches` | YES | |
| Launches (latest) | `GET /launches/latest` | YES | |
| Launches (next) | `GET /launches/next` | YES | |
| Launches (upcoming) | `GET /launches/upcoming` | YES | |
| Launches (past) | `GET /launches/past` | YES | |
| Rockets | `GET /rockets` | YES | |
| Crew | `GET /crew` | YES | |
| Starlink | `GET /starlink` | YES | |
| Launchpads | `GET /launchpads` | YES | |
| Landpads | `GET /landpads` | YES | |
| Payloads | `GET /payloads` | YES | |
| Capsules | `GET /capsules` | YES | |
| Cores | `GET /cores` | NO | First-stage booster cores (reuse history, landing outcomes) |
| Dragons | `GET /dragons` | NO | Dragon capsule vehicle specs (not individual capsule instances) |
| Ships | `GET /ships` | NO | Recovery ships (OCISLY, JRTI, GO Searcher, etc.) |
| Roadster | `GET /roadster` | NO | Elon's Tesla Roadster orbital data (niche but complete API coverage) |
| Company | `GET /company` | NO | SpaceX company info/stats (founding date, employees, etc.) |
| Query (POST) | `POST /{resource}/query` | NO | MongoDB-style query with population — available for all resources |

**Summary:** 5 missing resource endpoints (Cores, Dragons, Ships, Roadster, Company) plus the generic `POST /query` subsystem for all resources. `Cores` is high-value for tracking reusable booster status. `Ships` enables recovery fleet tracking.

---

## Batch 26 — Aviation

### 11. ADS-B Exchange — `intelligence_feeds/aviation/adsb_exchange/endpoints.rs`

**Current implementation:** 8 endpoints (AircraftNearLocation, AircraftByHex, AircraftByCallsign, AircraftByRegistration, AircraftByType, MilitaryAircraft, AircraftBySquawk, LaddAircraft)

Official docs: https://www.adsbexchange.com/version-2-api-wip/ and https://rapidapi.com/adsbx/api/adsbexchange-com1

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Near Location | `GET /v2/lat/{lat}/lon/{lon}/dist/{dist}/` | YES | |
| By ICAO Hex | `GET /v2/hex/{icao}/` | YES | |
| By Callsign | `GET /v2/callsign/{callsign}/` | YES | |
| By Registration | `GET /v2/registration/{reg}/` | YES | |
| By Type | `GET /v2/type/{type}/` | YES | |
| Military | `GET /v2/mil/` | YES | |
| By Squawk | `GET /v2/sqk/{squawk}/` | YES | |
| LADD Aircraft | `GET /v2/ladd/` | YES | Sensitive/privacy-protected aircraft |
| Global All Aircraft | Enterprise endpoint | NO | Full feed of all tracked aircraft — Enterprise tier only, not RapidAPI |
| Historical Data | Enterprise endpoint | NO | Historical position data — Enterprise tier only |

**Summary:** Coverage for RapidAPI tier is complete. Enterprise-tier endpoints (global feed, historical) are not applicable for standard access. Implementation is functionally complete for the accessible API.

---

### 12. AviationStack — `intelligence_feeds/aviation/aviationstack/endpoints.rs`

**Current implementation:** 7 endpoints (Flights, Airports, Airlines, AircraftTypes, Cities, Countries, Routes)

Official docs: https://aviationstack.com/documentation

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Flights (real-time) | `GET /flights` | YES | |
| Airports | `GET /airports` | YES | |
| Airlines | `GET /airlines` | YES | |
| Aircraft Types | `GET /aircraft_types` | YES | |
| Cities | `GET /cities` | YES | |
| Countries | `GET /countries` | YES | |
| Routes | `GET /routes` | YES | |
| Timetable | `GET /timetable` | NO | Airport departure/arrival timetables (scheduled flights) |
| Future Flights | `GET /flightsfuture` | NO | Flight data for dates >7 days in future |
| Airplanes | `GET /airplanes` | NO | Individual aircraft/airplane registration data |
| Taxes | `GET /taxes` | NO | Aviation tax zones (500+ zones) |
| Base URL (HTTPS) | Uses `http://` | BUG | API returns `http://` in docs but HTTPS is now supported on all plans — should use `https://` |

**Summary:** 4 missing endpoints. `Timetable` and `Flightsfuture` are particularly useful for scheduling and forward planning. `Airplanes` provides individual aircraft registration data distinct from `aircraft_types`. The base URL uses `http://` which is a bug — HTTPS is the correct protocol.

---

### 13. OpenSky — `intelligence_feeds/aviation/opensky/endpoints.rs`

**Current implementation:** 7 endpoints (StatesAll, StatesOwn, FlightsAll, FlightsAircraft, FlightsArrival, FlightsDeparture, TracksAll)

Official docs: https://openskynetwork.github.io/opensky-api/rest.html

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| States All | `GET /states/all` | YES | All aircraft state vectors |
| States Own | `GET /states/own` | YES | Own sensor data |
| Flights All | `GET /flights/all` | YES | Flights in time interval |
| Flights Aircraft | `GET /flights/aircraft` | YES | Flight history by ICAO24 |
| Flights Arrival | `GET /flights/arrival` | YES | Airport arrivals |
| Flights Departure | `GET /flights/departure` | YES | Airport departures |
| Tracks All | `GET /tracks/all` | YES | Aircraft trajectory waypoints |
| Airports | `GET /airports/{icao}` | NO | Airport metadata by ICAO4 code — recently added endpoint |
| Auth Change | Basic auth → OAuth2 | STALE | OpenSky migrated to OAuth2 client credentials — basic auth no longer works |

**Summary:** 1 missing endpoint (`/airports/{icao}`). More critically, the authentication implementation may be broken — OpenSky has migrated from username/password basic auth to OAuth2 client credentials flow. Tokens expire every 30 minutes. This is a functional blocker.

---

### 14. Wingbits — `intelligence_feeds/aviation/wingbits/endpoints.rs`

**Current implementation:** 2 endpoints (Details by ICAO24, BatchDetails)

Official docs: https://customer-api.wingbits.com/docs/ / https://wingbits.gitbook.io/developers

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Aircraft Details | `GET /api/wingbits/details/{icao24}` | YES | Single aircraft enrichment |
| Batch Details | `POST /api/wingbits/details/batch` | YES | Up to N aircraft in one request |
| Stream API | WebSocket stream | NO | Real-time aircraft data stream — mentioned in docs as "advanced integration" |
| Flight History | Possible endpoint | UNCLEAR | Historical flight data not confirmed in accessible docs |

**Summary:** 1 likely missing endpoint (Stream API). The main REST surface appears covered. Stream API is mentioned in Wingbits documentation for advanced integrations but not accessible without deeper auth.

---

### 15. FAA Status — `intelligence_feeds/faa_status/endpoints.rs`

**Current implementation:** 1 endpoint (`AirportStatusInfo` → `/api/airport-status-information`)

Official docs: https://nasstatus.faa.gov

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Airport Status Info | `GET /api/airport-status-information` | YES | NAS delays, closures, NOTAMs |
| FAA API Portal | https://api.faa.gov/s/ | NO | FAA's broader API portal — separate from NASSTATUS |
| ATIS/METAR-like data | Not exposed via NASSTATUS | N/A | Weather data via separate FAA systems |

**Summary:** The NASSTATUS API is genuinely a single-endpoint API. The `AirportStatusInfo` endpoint covers all NAS status data (delays, ground stops, closures). Coverage is complete for this source. The FAA has a separate broader API portal (`api.faa.gov`) that covers other data, but that would be a separate connector.

---

## Batch 27 — Maritime

### 16. AIS (Datalastic) — `intelligence_feeds/maritime/ais/endpoints.rs`

**Current implementation:** 7 endpoints (VesselFind, VesselInfo, VesselHistory, VesselPro, PortFind, PortInfo, FleetLiveMap)

Official docs: https://datalastic.com/api-reference/

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Vessel Find | `GET /vessel_find` | YES | Search by name/MMSI/IMO/callsign |
| Vessel Info | `GET /vessel_info` | YES | Detailed vessel info |
| Vessel History | `GET /vessel_history` | YES | Historical positions |
| Vessel Pro | `GET /vessel_pro` | YES | Premium: adds ETA, ATD, draft |
| Port Find | `GET /port_find` | YES | Port search |
| Port Info | `GET /port_info` | YES | Port details |
| Fleet Live Map | `GET /fleet_live_map` | YES | Live area positions |
| Vessel Bulk | `POST /vessel_bulk` | NO | Up to 100 vessels in one request — significant performance endpoint |
| Vessel In-Radius | `GET /vessel_inradius` | NO | All ships within a radius around a point — critical for chokepoint monitoring |
| Vessel Pro Estimated | `GET /vessel_pro_est` | NO | Estimated position when out of AIS range |
| Dry Dock Dates | Reports endpoint | NO | Planned/actual dry docking periods |
| Maritime Companies | Reports endpoint | NO | Company/operator data |
| Ship Casualties | Reports endpoint | NO | Historical casualty/incident data |
| Vessel Inspections | Reports endpoint | NO | Port state control inspection records |
| Sales & Demolitions | Reports endpoint | NO | Market activity data |
| Vessel Ownership | Reports endpoint | NO | Beneficial owner, operator, manager |
| Classification Society | Reports endpoint | NO | Classification society records |
| Vessel Engine | Reports endpoint | NO | Engine specifications |

**Summary:** 3 high-priority missing endpoints: `vessel_bulk` (batch efficiency), `vessel_inradius` (geospatial area search — essential for chokepoint monitoring), and `vessel_pro_est` (satellite-estimated positions when dark). The 8 maritime reports endpoints are lower priority but expand the data surface significantly.

---

### 17. AISStream — `intelligence_feeds/maritime/aisstream/endpoints.rs`

**Current implementation:** 1 WebSocket endpoint (Stream `wss://stream.aisstream.io/v0/stream`)

Official docs: https://aisstream.io/documentation

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| WebSocket Stream | `wss://stream.aisstream.io/v0/stream` | YES | Primary (and only) interface |
| Message Types Handled | `PositionReport`, `ShipStaticData` (inferred) | PARTIAL | 28+ message types exist — only 2-3 likely handled in parser |
| REST API | None | N/A | AISStream is WebSocket-only by design |

**Subscription filter coverage:**

| Filter Field | We Have? | Notes |
|-------------|----------|-------|
| `APIKey` | YES (auth) | Required |
| `BoundingBoxes` | YES (areas module) | Pre-defined chokepoints — good |
| `FiltersShipMMSI` | UNCLEAR | Not in endpoints.rs — check parser/connector |
| `FilterMessageTypes` | UNCLEAR | Not in endpoints.rs — check connector |

**Summary:** The single WebSocket endpoint is correctly modeled. The `areas` module with pre-defined chokepoints is a useful addition. Gap is at the message-type handling level in the parser, not the endpoint level. `FiltersShipMMSI` and `FilterMessageTypes` subscription fields may not be implemented in the connector.

---

### 18. IMF PortWatch — `intelligence_feeds/maritime/imf_portwatch/endpoints.rs`

**Current implementation:** 6 endpoints (Chokepoints, ChokepointStats, Ports, PortStats, TradeFlows, Disruptions)

Official docs: https://portwatch.imf.org

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Chokepoints List | `/portwatch/v1/chokepoints` | YES | 28 chokepoints |
| Chokepoint Stats | `/portwatch/v1/chokepoints/{id}/statistics` | YES | Per-chokepoint traffic stats |
| Ports List | `/portwatch/v1/ports` | YES | 2,033 ports |
| Port Stats | `/portwatch/v1/ports/{id}/statistics` | YES | Per-port traffic stats |
| Trade Flows | `/portwatch/v1/trade-flows` | YES | Global trade flow data |
| Disruptions | `/portwatch/v1/disruptions` | YES | Active disruptions |
| API Base URL | Uses `https://portwatch.imf.org/api` | UNVERIFIED | PortWatch may use ArcGIS GeoServices endpoints — official REST path not confirmed; platform uses ArcGIS infrastructure internally |

**Summary:** The 6 endpoints appear to cover the logical data categories available from PortWatch. However, the actual API paths (`/portwatch/v1/...`) are inferred/assumed — PortWatch's public interface uses ArcGIS GeoServices architecture and the true REST endpoint URLs are not officially published. This connector may have fundamentally incorrect base URL and paths. Needs verification against actual network traffic or official ArcGIS REST service endpoints.

---

### 19. NGA Warnings — `intelligence_feeds/maritime/nga_warnings/endpoints.rs`

**Current implementation:** 3 endpoints (BroadcastWarnings `/broadcast-warn`, NavigationalWarnings `/navwarn`, WarningById `/warn/{id}`)

Official docs: https://msi.nga.mil/NavWarnings and https://www.postman.com/api-evangelist/national-geospatial-intelligence-agency-nga

| Category | Endpoint | We Have? | Notes |
|----------|----------|----------|-------|
| Broadcast Warnings | `GET /broadcast-warn` | YES | NAVAREA/HYDROLANT/HYDROPAC active warnings |
| Navigational Warnings | `GET /navwarn` | YES | NAVAREAS warnings |
| Warning by ID | `GET /warn/{id}` | YES | Individual warning detail |
| ASAM Reports | `GET /asams` | NO | Anti-Shipping Activity Messages — piracy/hostile acts |
| MODU Positions | `GET /modu` | NO | Mobile Offshore Drilling Unit positions |
| World Port Index | `GET /world-port-index` | NO | NGA port database — 3,700+ ports worldwide |
| Notice to Mariners | `GET /publications/...` | NO | Official nautical chart corrections |
| Light List | `GET /publications/...` | NO | Navigational lights and aids to navigation |
| Chart Corrections | `GET /chart_corr` | NO | Hydrographic chart correction notices |

**Summary:** 3 high-priority missing endpoints. `ASAM` (anti-shipping activity messages) is extremely high value for conflict intelligence — tracks piracy and hostile maritime acts by region. `MODU` positions track offshore drilling units. `World Port Index` provides comprehensive port reference data. These are all documented in the NGA MSI Postman collection.

---

## Cross-Batch Summary

### Critical Gaps (Functional Impact)

| Connector | Gap | Impact |
|-----------|-----|--------|
| OpenSky | OAuth2 migration — basic auth broken | **BLOCKER** — connector likely non-functional |
| Space-Track | `cdm_public` (conjunction/collision data) | HIGH — core space safety data missing |
| Space-Track | Hardcoded query strings vs flexible builder | HIGH — architectural limitation |
| UCDP | `StateConflict` path wrong (should be `/ucdpprioconflict/`) | HIGH — wrong endpoint path |
| UCDP | Version numbers stale (`24.1` → `25.1`) | MEDIUM — returns older dataset |
| AviationStack | Uses `http://` base URL | MEDIUM — insecure, may fail |
| NASA | DONKI notifications, Mars Rover, Earth Imagery missing | MEDIUM — large coverage gap |
| Sentinel Hub | Catalog path incorrect (`/catalog/search` vs `/catalog/1.0.0/`) | MEDIUM — may not resolve |
| IMF PortWatch | Assumed endpoint paths unverified | MEDIUM — may be entirely wrong |

### Endpoint Count Summary

| Connector | Implemented | Missing (Confirmed) | Notes |
|-----------|-------------|---------------------|-------|
| ACLED | 1 | 2 (cast, deleted) | Minimal API by design |
| GDELT | 4 | 0 | Complete |
| ReliefWeb | 6 | 3 (blog, book, references) | + should upgrade to v2 |
| UCDP | 5 | 1 (dyadic) | + path bug + stale versions |
| UNHCR | 5 | 1 (asylum-applications) | |
| Launch Library 2 | 10 | 6 (launcher, pad, location, expedition, docking_event, payload) | + stale API version |
| NASA | 9 | 11+ (DONKI suite gaps, Mars, Earth) | Significant gaps |
| Sentinel Hub | 4 | 7 (batch, OGC, collections) | + path bug |
| Space-Track | 7 | 7 (cdm_public, gp_history, boxscore, etc.) | + architecture issue |
| SpaceX | 12 | 5 (cores, dragons, ships, roadster, company) | |
| ADS-B Exchange | 8 | 0 (enterprise endpoints N/A) | Complete for accessible tier |
| AviationStack | 7 | 4 (timetable, flightsfuture, airplanes, taxes) | + http bug |
| OpenSky | 7 | 1 (airports) | + auth BLOCKER |
| Wingbits | 2 | 1 (stream) | Minimal API |
| FAA Status | 1 | 0 | Single-endpoint API, complete |
| AIS (Datalastic) | 7 | 3+ priority (bulk, inradius, pro_est) | + 8 reports endpoints |
| AISStream | 1 (WS) | 0 (WS-only API) | Parser coverage gap |
| IMF PortWatch | 6 | 0 (confirmed) | Paths unverified |
| NGA Warnings | 3 | 3 (asam, modu, world-port-index) | ASAM is high priority |

---

## Sources

- [ACLED API Documentation](https://acleddata.com/acled-api-documentation)
- [ACLED Endpoint Reference](https://acleddata.com/api-documentation/acled-endpoint)
- [GDELT DOC 2.0 API](https://blog.gdeltproject.org/gdelt-doc-2-0-api-debuts/)
- [GDELT GEO 2.0 API](https://blog.gdeltproject.org/gdelt-geo-2-0-api-debuts/)
- [GDELT TV 2.0 API](https://blog.gdeltproject.org/gdelt-2-0-television-api-debuts/)
- [GDELT Context 2.0 API](https://blog.gdeltproject.org/announcing-the-gdelt-context-2-0-api/)
- [ReliefWeb API Endpoints](https://apidoc.reliefweb.int/endpoints)
- [UCDP API Documentation](https://ucdp.uu.se/apidocs/)
- [UNHCR Refugee Statistics API](https://api.unhcr.org/docs/refugee-statistics.html)
- [Launch Library 2 API Docs](https://ll.thespacedevs.com/docs)
- [Launch Library 2 v2.3.0 Changelog](https://www.patreon.com/posts/launch-library-2-112553005)
- [NASA API Portal](https://api.nasa.gov)
- [Sentinel Hub API Overview](https://docs.sentinel-hub.com/api/latest/api/overview/)
- [Sentinel Hub Catalog API](https://docs.sentinel-hub.com/api/latest/api/catalog/)
- [Sentinel Hub Processing API](https://docs.sentinel-hub.com/api/latest/api/process/)
- [Sentinel Hub Statistical API](https://docs.sentinel-hub.com/api/latest/api/statistical/)
- [Space-Track.org Documentation](https://www.space-track.org/documentation)
- [SpaceX API GitHub](https://github.com/r-spacex/SpaceX-API)
- [ADS-B Exchange v2 API Fields](https://www.adsbexchange.com/version-2-api-wip/)
- [ADS-B Exchange RapidAPI](https://rapidapi.com/adsbx/api/adsbexchange-com1)
- [AviationStack Documentation](https://aviationstack.com/documentation)
- [OpenSky REST API Docs](https://openskynetwork.github.io/opensky-api/rest.html)
- [Wingbits Developer Docs](https://wingbits.gitbook.io/developers)
- [Wingbits Customer API Reference](https://customer-api.wingbits.com/docs/)
- [FAA NASSTATUS API](https://nasstatus.faa.gov)
- [Datalastic AIS API Reference](https://datalastic.com/api-reference/)
- [AISStream.io Documentation](https://aisstream.io/documentation)
- [IMF PortWatch Platform](https://portwatch.imf.org/)
- [IMF PortWatch Data & Methodology](https://portwatch.imf.org/pages/data-and-methodology)
- [NGA Maritime Safety Information](https://msi.nga.mil/home)
- [NGA Navigational Warnings](https://msi.nga.mil/NavWarnings)
- [NGA MSI REST API (Postman)](https://www.postman.com/api-evangelist/national-geospatial-intelligence-agency-nga/documentation/ak1v6h5/maritime-safety-information-rest-api)
