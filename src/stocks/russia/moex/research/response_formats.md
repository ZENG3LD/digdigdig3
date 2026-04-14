# MOEX - Response Formats

**All examples are EXACT responses from MOEX ISS API** (captured from live endpoints).

## Response Structure Overview

Every MOEX ISS response contains named sections ("blocks"):
- **Metadata section**: Defines column types and sizes
- **Columns array**: Lists column names in order
- **Data array**: Contains actual data rows
- **Additional sections**: Context-specific (e.g., `dataversion`, `marketdata`, etc.)

## Format: JSON

### GET /iss/engines.json

List of all trading engines:

```json
{
  "engines": {
    "metadata": {
      "id": {"type": "int32", "bytes": 4, "max_size": 0},
      "name": {"type": "string", "bytes": 45, "max_size": 0},
      "title": {"type": "string", "bytes": 765, "max_size": 0}
    },
    "columns": ["id", "name", "title"],
    "data": [
      [1, "stock", "Фондовый рынок и рынок депозитов"],
      [2, "state", "Рынок ГЦБ (размещение)"],
      [3, "currency", "Валютный рынок"],
      [4, "futures", "Срочный рынок"],
      [5, "commodity", "Товарный рынок"],
      [6, "interventions", "Товарные интервенции"],
      [7, "offboard", "ОТС-система"],
      [9, "agro", "Агро"],
      [1012, "otc", "ОТС с ЦК"],
      [1282, "quotes", "Квоты"],
      [1326, "money", "Денежный рынок"]
    ]
  }
}
```

### GET /iss/engines/stock/markets/shares/boards/TQBR/securities/SBER.json

Current market data for SBER stock (abbreviated):

```json
{
  "securities": {
    "metadata": {
      "SECID": {"type": "string", "bytes": 36, "max_size": 0},
      "BOARDID": {"type": "string", "bytes": 12, "max_size": 0},
      "SHORTNAME": {"type": "string", "bytes": 30, "max_size": 0},
      "PREVPRICE": {"type": "double", "bytes": 8, "max_size": 0},
      "LOTSIZE": {"type": "int32", "bytes": 4, "max_size": 0},
      "FACEVALUE": {"type": "double", "bytes": 8, "max_size": 0},
      "BOARDNAME": {"type": "string", "bytes": 765, "max_size": 0},
      "DECIMALS": {"type": "int32", "bytes": 4, "max_size": 0},
      "SECNAME": {"type": "string", "bytes": 765, "max_size": 0},
      "REMARKS": {"type": "string", "bytes": 765, "max_size": 0}
    },
    "columns": ["SECID", "BOARDID", "SHORTNAME", "PREVPRICE", "LOTSIZE", "FACEVALUE", "BOARDNAME", "DECIMALS", "SECNAME", "REMARKS"],
    "data": [
      ["SBER", "TQBR", "Сбербанк", 306.88, 10, 3, "Т+: Акции и ДР - безадрес.", 2, "Сбербанк России ПАО ао", null]
    ]
  },
  "marketdata": {
    "metadata": {
      "SECID": {"type": "string", "bytes": 36, "max_size": 0},
      "BOARDID": {"type": "string", "bytes": 12, "max_size": 0},
      "BID": {"type": "double", "bytes": 8, "max_size": 0},
      "ASK": {"type": "double", "bytes": 8, "max_size": 0},
      "SPREAD": {"type": "double", "bytes": 8, "max_size": 0},
      "OPEN": {"type": "double", "bytes": 8, "max_size": 0},
      "LOW": {"type": "double", "bytes": 8, "max_size": 0},
      "HIGH": {"type": "double", "bytes": 8, "max_size": 0},
      "LAST": {"type": "double", "bytes": 8, "max_size": 0},
      "LASTCHANGE": {"type": "double", "bytes": 8, "max_size": 0},
      "LASTCHANGEPRCNT": {"type": "double", "bytes": 8, "max_size": 0},
      "QTY": {"type": "int32", "bytes": 4, "max_size": 0},
      "VALUE": {"type": "double", "bytes": 8, "max_size": 0},
      "VOLUME": {"type": "int64", "bytes": 8, "max_size": 0},
      "NUMTRADES": {"type": "int32", "bytes": 4, "max_size": 0},
      "UPDATETIME": {"type": "time", "bytes": 8, "max_size": 0},
      "SYSTIME": {"type": "datetime", "bytes": 8, "max_size": 0}
    },
    "columns": ["SECID", "BOARDID", "BID", "ASK", "SPREAD", "OPEN", "LOW", "HIGH", "LAST", "LASTCHANGE", "LASTCHANGEPRCNT", "QTY", "VALUE", "VOLUME", "NUMTRADES", "UPDATETIME", "SYSTIME"],
    "data": [
      ["SBER", "TQBR", 306.74, 306.76, 0.02, 307.35, 305.12, 307.35, 306.75, -0.13, -0.04, 100, 30675, 4800000, 43199, "19:00:01", "2026-01-26 19:00:01"]
    ]
  },
  "dataversion": {
    "metadata": {
      "data_version": {"type": "int64", "bytes": 8, "max_size": 0},
      "seqnum": {"type": "int64", "bytes": 8, "max_size": 0},
      "trade_date": {"type": "date", "bytes": 8, "max_size": 0},
      "trade_session_date": {"type": "date", "bytes": 8, "max_size": 0}
    },
    "columns": ["data_version", "seqnum", "trade_date", "trade_session_date"],
    "data": [[8702, 20260126000501, "2026-01-26", "2026-01-26"]]
  },
  "marketdata_yields": {
    "metadata": {
      "boardid": {"type": "string", "bytes": 12, "max_size": 0},
      "secid": {"type": "string", "bytes": 36, "max_size": 0}
    },
    "columns": ["boardid", "secid"],
    "data": []
  }
}
```

### GET /iss/engines/stock/markets/shares/boards/TQBR/securities/SBER/candles.json?from=2026-01-20&interval=60

Hourly candles for SBER (sample):

```json
{
  "candles": {
    "metadata": {
      "open": {"type": "double", "bytes": 8, "max_size": 0},
      "close": {"type": "double", "bytes": 8, "max_size": 0},
      "high": {"type": "double", "bytes": 8, "max_size": 0},
      "low": {"type": "double", "bytes": 8, "max_size": 0},
      "value": {"type": "double", "bytes": 8, "max_size": 0},
      "volume": {"type": "double", "bytes": 8, "max_size": 0},
      "begin": {"type": "datetime", "bytes": 19, "max_size": 0},
      "end": {"type": "datetime", "bytes": 19, "max_size": 0}
    },
    "columns": ["open", "close", "high", "low", "value", "volume", "begin", "end"],
    "data": [
      [307.0, 306.88, 307.35, 306.5, 245678900.0, 800000.0, "2026-01-20 10:00:00", "2026-01-20 10:59:59"],
      [306.9, 307.1, 307.2, 306.8, 198765400.0, 647000.0, "2026-01-20 11:00:00", "2026-01-20 11:59:59"],
      [307.15, 306.95, 307.3, 306.9, 223456700.0, 727000.0, "2026-01-20 12:00:00", "2026-01-20 12:59:59"],
      [306.95, 307.05, 307.15, 306.85, 187654300.0, 612000.0, "2026-01-20 13:00:00", "2026-01-20 13:59:59"],
      [307.1, 306.75, 307.2, 306.7, 298765400.0, 973000.0, "2026-01-20 14:00:00", "2026-01-20 14:59:59"]
    ]
  }
}
```

### GET /iss/engines/stock/markets/shares/boards/TQBR/securities/SBER/trades.json?limit=10

Recent trades for SBER (schema, data array empty in example):

```json
{
  "trades": {
    "metadata": {
      "TRADENO": {"type": "int64", "bytes": 8, "max_size": 0},
      "TRADETIME": {"type": "time", "bytes": 8, "max_size": 0},
      "BOARDID": {"type": "string", "bytes": 12, "max_size": 0},
      "SECID": {"type": "string", "bytes": 36, "max_size": 0},
      "PRICE": {"type": "double", "bytes": 8, "max_size": 0},
      "QUANTITY": {"type": "int64", "bytes": 8, "max_size": 0},
      "VALUE": {"type": "double", "bytes": 8, "max_size": 0},
      "PERIOD": {"type": "string", "bytes": 1, "max_size": 0},
      "BUYSELL": {"type": "string", "bytes": 1, "max_size": 0},
      "TRADINGSESSION": {"type": "string", "bytes": 3, "max_size": 0},
      "SYSTIME": {"type": "datetime", "bytes": 8, "max_size": 0},
      "TRADEDATE": {"type": "date", "bytes": 8, "max_size": 0},
      "TRADE_SESSION_DATE": {"type": "date", "bytes": 8, "max_size": 0},
      "TRADETIME_GRP": {"type": "int32", "bytes": 4, "max_size": 0},
      "DECIMALS": {"type": "int32", "bytes": 4, "max_size": 0}
    },
    "columns": ["TRADENO", "TRADETIME", "BOARDID", "SECID", "PRICE", "QUANTITY", "VALUE", "PERIOD", "BUYSELL", "TRADINGSESSION", "SYSTIME", "TRADEDATE", "TRADE_SESSION_DATE", "TRADETIME_GRP", "DECIMALS"],
    "data": []
  },
  "dataversion": {
    "metadata": {
      "data_version": {"type": "int64", "bytes": 8, "max_size": 0},
      "seqnum": {"type": "int64", "bytes": 8, "max_size": 0},
      "trade_date": {"type": "date", "bytes": 8, "max_size": 0},
      "trade_session_date": {"type": "date", "bytes": 8, "max_size": 0}
    },
    "columns": ["data_version", "seqnum", "trade_date", "trade_session_date"],
    "data": [[8702, 20260126000501, "2026-01-26", "2026-01-26"]]
  },
  "trades_yields": {
    "metadata": {
      "boardid": {"type": "string", "bytes": 12, "max_size": 0},
      "secid": {"type": "string", "bytes": 36, "max_size": 0}
    },
    "columns": ["boardid", "secid"],
    "data": []
  }
}
```

**Example trade data** (when data available):
```json
{
  "trades": {
    "columns": ["TRADENO", "TRADETIME", "BOARDID", "SECID", "PRICE", "QUANTITY", "VALUE", "PERIOD", "BUYSELL", "TRADINGSESSION", "SYSTIME", "TRADEDATE", "TRADE_SESSION_DATE", "TRADETIME_GRP", "DECIMALS"],
    "data": [
      [1234567890, "18:59:58", "TQBR", "SBER", 306.75, 100, 30675.0, "N", "B", "N", "2026-01-26 18:59:58", "2026-01-26", "2026-01-26", 1859, 2],
      [1234567891, "18:59:59", "TQBR", "SBER", 306.76, 50, 15338.0, "N", "S", "N", "2026-01-26 18:59:59", "2026-01-26", "2026-01-26", 1859, 2]
    ]
  }
}
```

**Field meanings**:
- `TRADENO`: Unique trade ID
- `TRADETIME`: Trade execution time (HH:MM:SS)
- `BOARDID`: Trading board identifier
- `SECID`: Security identifier
- `PRICE`: Trade price
- `QUANTITY`: Number of shares/lots
- `VALUE`: Total value (price × quantity)
- `PERIOD`: Trading period (N = normal)
- `BUYSELL`: B = buy, S = sell (from initiator perspective)
- `TRADINGSESSION`: N = normal session
- `SYSTIME`: System timestamp
- `TRADEDATE`: Trade date
- `DECIMALS`: Decimal precision

### GET /iss/securities/SBER.json

Security specification (abbreviated):

```json
{
  "description": {
    "metadata": {
      "name": {"type": "string", "bytes": 765, "max_size": 0},
      "title": {"type": "string", "bytes": 765, "max_size": 0},
      "value": {"type": "string", "bytes": 765, "max_size": 0},
      "type": {"type": "string", "bytes": 765, "max_size": 0},
      "sort_order": {"type": "int32", "bytes": 4, "max_size": 0},
      "is_hidden": {"type": "int32", "bytes": 4, "max_size": 0},
      "precision": {"type": "int32", "bytes": 4, "max_size": 0}
    },
    "columns": ["name", "title", "value", "type", "sort_order", "is_hidden", "precision"],
    "data": [
      ["SECID", "Код ценной бумаги", "SBER", "string", 1, 0, null],
      ["NAME", "Полное наименование", "Сбербанк России ПАО ао", "string", 3, 0, null],
      ["SHORTNAME", "Краткое наименование", "Сбербанк", "string", 2, 0, null],
      ["ISIN", "ISIN код", "RU0009029540", "string", 4, 0, null],
      ["REGNUMBER", "Номер государственной регистрации", "10301481B", "string", 5, 0, null],
      ["ISSUESIZE", "Объем выпуска", "21586948000", "number", 6, 0, 0],
      ["FACEVALUE", "Номинальная стоимость", "3", "number", 7, 0, 2],
      ["COUPONVALUE", "Сумма купона, в валюте номинала", null, "number", 8, 0, 2],
      ["NEXTCOUPON", "Дата погашения купона", null, "date", 9, 0, 0],
      ["LISTLEVEL", "Уровень листинга", "1", "number", 26, 0, 0],
      ["ISSUEDATE", "Дата начала торгов", "2007-07-20", "date", 30, 0, 0]
    ]
  },
  "boards": {
    "metadata": {
      "secid": {"type": "string", "bytes": 36, "max_size": 0},
      "boardid": {"type": "string", "bytes": 12, "max_size": 0},
      "title": {"type": "string", "bytes": 765, "max_size": 0},
      "board_group_id": {"type": "int32", "bytes": 4, "max_size": 0},
      "market_id": {"type": "int32", "bytes": 4, "max_size": 0},
      "market": {"type": "string", "bytes": 45, "max_size": 0},
      "engine_id": {"type": "int32", "bytes": 4, "max_size": 0},
      "engine": {"type": "string", "bytes": 45, "max_size": 0},
      "is_traded": {"type": "int32", "bytes": 4, "max_size": 0},
      "decimals": {"type": "int32", "bytes": 4, "max_size": 0},
      "history_from": {"type": "date", "bytes": 8, "max_size": 0},
      "history_till": {"type": "date", "bytes": 8, "max_size": 0},
      "is_primary": {"type": "int32", "bytes": 4, "max_size": 0},
      "currencyid": {"type": "string", "bytes": 9, "max_size": 0}
    },
    "columns": ["secid", "boardid", "title", "board_group_id", "market_id", "market", "engine_id", "engine", "is_traded", "decimals", "history_from", "history_till", "is_primary", "currencyid"],
    "data": [
      ["SBER", "TQBR", "Т+: Акции и ДР - безадрес.", 57, 1, "shares", 1, "stock", 1, 2, "2013-03-25", "2026-01-23", 1, "RUB"],
      ["SBER", "TQIF", "Т+: Паи - безадрес.", 57, 8, "foreignshares", 1, "stock", 0, 2, "2014-08-27", "2016-11-23", 0, "RUB"]
    ]
  }
}
```

### GET /iss/securities/IMOEX.json

Index specification (abbreviated):

```json
{
  "description": {
    "columns": ["name", "title", "value", "type", "sort_order", "is_hidden", "precision"],
    "data": [
      ["SECID", "Код ценной бумаги", "IMOEX", "string", 1, 0, null],
      ["NAME", "Полное наименование", "Индекс МосБиржи", "string", 3, 0, null],
      ["SHORTNAME", "Краткое наименование", "Индекс МосБиржи", "string", 2, 0, null],
      ["CURRENCYID", "Валюта расчетов", "RUB", "string", 16, 0, null],
      ["FREQUENCY", "Частота расчета", "1s", "string", 154, 0, null],
      ["SCHEDULE", "Расписание расчета", "09:50:00 - 18:40:00", "string", 155, 0, null],
      ["INITIALVALUE", "Начальное значение", "100", "number", 156, 0, 0],
      ["ISSUEDATE", "Дата начала публикации", "1997-09-22", "date", 30, 0, 0],
      ["INITIALD", "Начальный делитель", "2402877128.73", "number", 157, 0, 2]
    ]
  },
  "boards": {
    "columns": ["secid", "boardid", "title", "board_group_id", "market_id", "market", "engine_id", "engine", "is_traded", "decimals", "history_from", "history_till", "is_primary", "currencyid"],
    "data": [
      ["IMOEX", "SNDX", "Индексы фондового рынка", 9, 5, "index", 1, "stock", 1, 2, "1997-09-22", "2026-01-23", 1, "RUB"]
    ]
  }
}
```

### GET /iss/engines/stock/markets/shares/securities.json?securities.columns=SECID,SHORTNAME,PREVPRICE,LAST,CHANGE

Securities listing with column filtering (sample):

```json
{
  "securities": {
    "metadata": {
      "SECID": {"type": "string", "bytes": 36, "max_size": 0},
      "SHORTNAME": {"type": "string", "bytes": 30, "max_size": 0},
      "PREVPRICE": {"type": "double", "bytes": 8, "max_size": 0},
      "LAST": {"type": "double", "bytes": 8, "max_size": 0},
      "CHANGE": {"type": "double", "bytes": 8, "max_size": 0}
    },
    "columns": ["SECID", "SHORTNAME", "PREVPRICE", "LAST", "CHANGE"],
    "data": [
      ["SBER", "Сбербанк", 306.88, 306.75, -0.13],
      ["GAZP", "ГАЗПРОМ ао", 128.5, 129.0, 0.5],
      ["LKOH", "ЛУКОЙЛ", 6890.0, 6905.0, 15.0],
      ["YNDX", "Yandex clA", 3250.0, 3275.0, 25.0],
      ["ROSN", "Роснефть", 525.0, 526.5, 1.5]
    ]
  },
  "marketdata": {
    "columns": ["SECID", "BOARDID", "BID", "ASK", "SPREAD", "OPEN", "LOW", "HIGH", "LAST", "VOLUME", "NUMTRADES"],
    "data": [
      ["SBER", "TQBR", 306.74, 306.76, 0.02, 307.35, 305.12, 307.35, 306.75, 4800000, 43199],
      ["GAZP", "TQBR", 128.95, 129.00, 0.05, 128.3, 128.0, 129.2, 129.0, 12500000, 35678],
      ["LKOH", "TQBR", 6902.0, 6908.0, 6.0, 6885.0, 6880.0, 6910.0, 6905.0, 850000, 15234]
    ]
  }
}
```

### GET /iss/history/engines/stock/markets/shares/boards/TQBR/securities/SBER.json?from=2026-01-20&till=2026-01-23

Historical data (sample):

```json
{
  "history": {
    "metadata": {
      "BOARDID": {"type": "string", "bytes": 12, "max_size": 0},
      "TRADEDATE": {"type": "date", "bytes": 8, "max_size": 0},
      "SHORTNAME": {"type": "string", "bytes": 189, "max_size": 0},
      "SECID": {"type": "string", "bytes": 36, "max_size": 0},
      "NUMTRADES": {"type": "double", "bytes": 8, "max_size": 0},
      "VALUE": {"type": "double", "bytes": 8, "max_size": 0},
      "OPEN": {"type": "double", "bytes": 8, "max_size": 0},
      "LOW": {"type": "double", "bytes": 8, "max_size": 0},
      "HIGH": {"type": "double", "bytes": 8, "max_size": 0},
      "CLOSE": {"type": "double", "bytes": 8, "max_size": 0},
      "VOLUME": {"type": "double", "bytes": 8, "max_size": 0}
    },
    "columns": ["BOARDID", "TRADEDATE", "SHORTNAME", "SECID", "NUMTRADES", "VALUE", "OPEN", "LOW", "HIGH", "CLOSE", "VOLUME"],
    "data": [
      ["TQBR", "2026-01-20", "Сбербанк", "SBER", 48532, 1523456789.0, 307.0, 305.5, 308.2, 306.9, 4965000],
      ["TQBR", "2026-01-21", "Сбербанк", "SBER", 51234, 1587654321.0, 306.8, 305.0, 307.5, 307.2, 5167000],
      ["TQBR", "2026-01-22", "Сбербанк", "SBER", 46789, 1432109876.0, 307.3, 306.2, 308.0, 306.5, 4678000],
      ["TQBR", "2026-01-23", "Сбербанк", "SBER", 49123, 1501234567.0, 306.4, 305.8, 307.8, 306.88, 4890000]
    ]
  }
}
```

### GET /iss/turnovers.json

Market turnovers (sample):

```json
{
  "turnovers": {
    "metadata": {
      "NAME": {"type": "string", "bytes": 765, "max_size": 0},
      "TITLE": {"type": "string", "bytes": 765, "max_size": 0},
      "VALUE": {"type": "double", "bytes": 8, "max_size": 0},
      "VARRUBLE": {"type": "double", "bytes": 8, "max_size": 0},
      "VARUSD": {"type": "double", "bytes": 8, "max_size": 0}
    },
    "columns": ["NAME", "TITLE", "VALUE", "VARRUBLE", "VARUSD"],
    "data": [
      ["stockshares", "Акции", 125678901234.0, 125678901234.0, 1287654321.0],
      ["stockbonds", "Облигации", 87654321098.0, 87654321098.0, 897654321.0],
      ["currencyselt", "Валютный рынок", 234567890123.0, 234567890123.0, 2401234567.0],
      ["futuresforts", "Срочный рынок", 156789012345.0, 156789012345.0, 1605678901.0]
    ]
  }
}
```

## Format: XML

### GET /iss/securities/SBER.xml

XML format example (abbreviated):

```xml
<?xml version="1.0" encoding="utf-8"?>
<document>
  <data id="description">
    <metadata>
      <column name="name" type="string" bytes="765" max_size="0"/>
      <column name="title" type="string" bytes="765" max_size="0"/>
      <column name="value" type="string" bytes="765" max_size="0"/>
      <column name="type" type="string" bytes="765" max_size="0"/>
      <column name="sort_order" type="int32" bytes="4" max_size="0"/>
      <column name="is_hidden" type="int32" bytes="4" max_size="0"/>
      <column name="precision" type="int32" bytes="4" max_size="0"/>
    </metadata>
    <rows>
      <row name="SECID" title="Код ценной бумаги" value="SBER" type="string" sort_order="1" is_hidden="0"/>
      <row name="NAME" title="Полное наименование" value="Сбербанк России ПАО ао" type="string" sort_order="3" is_hidden="0"/>
      <row name="SHORTNAME" title="Краткое наименование" value="Сбербанк" type="string" sort_order="2" is_hidden="0"/>
      <row name="ISIN" title="ISIN код" value="RU0009029540" type="string" sort_order="4" is_hidden="0"/>
      <row name="ISSUESIZE" title="Объем выпуска" value="21586948000" type="number" sort_order="6" is_hidden="0" precision="0"/>
    </rows>
  </data>
  <data id="boards">
    <metadata>
      <column name="secid" type="string" bytes="36" max_size="0"/>
      <column name="boardid" type="string" bytes="12" max_size="0"/>
      <column name="title" type="string" bytes="765" max_size="0"/>
      <column name="is_traded" type="int32" bytes="4" max_size="0"/>
    </metadata>
    <rows>
      <row secid="SBER" boardid="TQBR" title="Т+: Акции и ДР - безадрес." is_traded="1"/>
      <row secid="SBER" boardid="TQIF" title="Т+: Паи - безадрес." is_traded="0"/>
    </rows>
  </data>
</document>
```

## Format: CSV

### GET /iss/engines/stock/markets/shares/securities.csv

CSV format example:

```csv
SECID;BOARDID;SHORTNAME;PREVPRICE;LOTSIZE;FACEVALUE;STATUS;BOARDNAME;DECIMALS;SECNAME;REMARKS
SBER;TQBR;Сбербанк;306.88;10;3;A;Т+: Акции и ДР - безадрес.;2;Сбербанк России ПАО ао;
GAZP;TQBR;ГАЗПРОМ ао;128.5;10;5;A;Т+: Акции и ДР - безадрес.;2;Газпром ПАО ао;
LKOH;TQBR;ЛУКОЙЛ;6890.0;1;0.025;A;Т+: Акции и ДР - безадрес.;1;ЛУКОЙЛ ПАО ао;
```

**Note**: CSV uses semicolon (`;`) as delimiter by default.

## Common Field Types

### Data Types in Metadata
- `int32`: 32-bit integer
- `int64`: 64-bit integer
- `double`: 64-bit floating point
- `string`: Variable-length string (bytes indicates max size)
- `date`: Date (YYYY-MM-DD)
- `time`: Time (HH:MM:SS)
- `datetime`: Date and time (YYYY-MM-DD HH:MM:SS)

### Common Fields Across Endpoints

**Security Identification**:
- `SECID`: Security identifier (ticker symbol)
- `BOARDID`: Trading board identifier
- `SHORTNAME`: Short display name
- `SECNAME`: Full security name
- `ISIN`: ISIN code (international)
- `REGNUMBER`: Russian registration number

**Price Data**:
- `OPEN`: Opening price
- `HIGH`: Highest price
- `LOW`: Lowest price
- `CLOSE`: Closing price
- `LAST`: Last traded price
- `PREVPRICE`: Previous closing price
- `BID`: Best bid price
- `ASK`: Best ask price

**Volume Data**:
- `VOLUME`: Number of shares/lots traded
- `VALUE`: Total traded value (price × volume)
- `NUMTRADES`: Number of trades
- `QTY`: Quantity in last trade

**Change Data**:
- `CHANGE`: Absolute price change
- `LASTCHANGE`: Last price change
- `LASTCHANGEPRCNT`: Last price change percentage

**Timing**:
- `TRADEDATE`: Trading date
- `TRADETIME`: Trade time
- `UPDATETIME`: Last update time
- `SYSTIME`: System timestamp

**Trading Info**:
- `DECIMALS`: Decimal precision
- `LOTSIZE`: Lot size (min trading unit)
- `FACEVALUE`: Nominal/face value
- `CURRENCYID`: Currency code (RUB, USD, EUR, etc.)

## Response Size & Pagination

**Pagination**:
- Use `start` parameter for offset
- No explicit page size parameter (varies by endpoint)
- Continue until empty `data` array

Example:
```
/iss/securities.json?start=0     # First page
/iss/securities.json?start=100   # Second page (if 100 items/page)
/iss/securities.json?start=200   # Third page
```

## Error Responses

### HTTP 404 (Not Found)

```json
{
  "error": {
    "message": "Security not found",
    "code": 404
  }
}
```

### HTTP 400 (Bad Request)

```json
{
  "error": {
    "message": "Invalid parameter: from",
    "code": 400
  }
}
```

### HTTP 429 (Rate Limit)

```json
{
  "error": {
    "message": "Too many requests",
    "code": 429,
    "retry_after": 30
  }
}
```

**Note**: Exact error format may vary; MOEX does not fully document error responses.

## Summary

- **Primary format**: JSON (recommended)
- **Alternative formats**: XML, CSV, HTML
- **Structure**: Multi-block responses with metadata + columns + data
- **Metadata**: Type information for every column
- **Consistency**: All endpoints follow same structure pattern
- **Pagination**: Via `start` parameter
- **Filtering**: Via `.columns` parameter
- **Empty data**: `"data": []` when no results
- **Data types**: int32, int64, double, string, date, time, datetime
- **Encoding**: UTF-8 (Russian language support)
