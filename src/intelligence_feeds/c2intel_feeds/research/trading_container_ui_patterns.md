# Professional Trading Container UI/UX Patterns Research

**Date**: 2026-02-16
**Research Focus**: DOM + Cluster composite panel layouts in professional trading platforms

## Executive Summary

Professional trading platforms follow consistent layout patterns when combining Depth of Market (DOM) with order flow visualization tools. The **DOM ladder is ALWAYS the center anchor element**, with cluster/footprint charts on the left, liquidity heatmaps extending left from the DOM, and trade feeds/volume profiles on the right. This research examines ATAS, Quantower, Sierra Chart, Bookmap, Tiger Trade, and CScalp implementations.

---

## 1. ATAS (Advanced Time and Sales)

### Core Layout Structure

**Center**: Cluster/Footprint chart (primary visualization)
**Right Side**: Depth of Market (DOM) indicator displayed as horizontal bars
- Red bars (above price) = sell limit orders
- Green bars (below price) = buy limit orders

**Panel Organization**:
- Main chart area shows cluster charts with 25+ visualization variants
- DOM appears as overlay or sidebar (configurable)
- Settings menu positioned on right side for quick mode switching

### Cluster Chart Modes

ATAS provides five primary cluster modes:
1. **Volume** - total volume at each price level
2. **Trades** - number of trades executed
3. **Time** - time spent at each price level
4. **Bid x Ask** - buyer/seller activity split (7 variants)
5. **Delta** - net difference between buying/selling pressure

### Smart Tape (Time & Sales)

The **Smart Tape** is ATAS's aggregated trade feed:

**Display Format** (Vertical tape):
- Column layout showing: Time | Price | Direction (++ / ---) | Volume | Bid | Ask | Bid Size | Ask Size
- Trades scroll vertically (newest at top or bottom, configurable)
- Large trades highlighted based on threshold settings
- "Above Ask and Below Bid" coloring for off-market trades

**Aggregation Feature**:
- Groups fragmented ticks into complete market orders
- Example: 100-lot order shown as single 100-lot print instead of 100 separate 1-lot prints
- Aggregation period is user-configurable

**Integration**:
- Can be docked alongside cluster charts and DOM
- Freeze function (snowflake button) for analysis
- Historical playback mode available

### Key Features

- Over 70 built-in order flow tools
- Smart DOM shows liquidity levels and resting orders
- "Big Trades" filter highlights institutional activity
- Multi-dimensional profiles (Volume Profile, Delta Profile, etc.)
- Visual imbalance highlighting in Bid x Ask mode

**Sources**:
- [ATAS Cluster Chart Anatomy](https://atas.net/atas-possibilities/cluster-charts-footprint/cluster-chart-footprint-anatomy/)
- [ATAS Cluster Chart Functionality](https://atas.net/blog/cluster-chart-functionality/)
- [ATAS Smart Tape Description](https://help.atas.net/en/support/solutions/articles/72000602608-description-setting-templates-smart-tape-)
- [Setting Up Smart Tape](https://atas.net/trading-preparation/smart-tape/)

---

## 2. Quantower

### DOM Surface Panel

The **DOM Surface** is Quantower's liquidity heatmap + execution visualization:

**Layout Structure**:

**Center**: Heatmap visualization of order book changes over time
- Time flows horizontally (left = past, right = present)
- Price on vertical axis (synced with DOM Trader if open)
- Three coloring modes: Monochrome, Bichrome, Heatmap

**Right Side**: Three histogram columns
1. **Size** - volume of limit orders at each price level
2. **Cumulative Size** - summed volumes showing market dominance
3. **Order Book Imbalance** - percentage differential (buyers vs sellers)

**Execution Visualization**:
- **Red circles** = sell order executions
- **Green circles** = buy order executions
- Circle size = order volume
- Hover shows "actual size of executed order"

**Top Control Bar**:
- Quick settings for DOM levels (depth)
- Contrast adjustment to highlight high-liquidity zones
- Auto-centering toggle to follow price

**Visual Reference Lines**:
- Last Trade line (current price)
- Bid/Ask lines (customizable colors)

### DOM Trader Panel

The **DOM Trader** is Quantower's traditional order entry interface:

**Layout Structure**:

**Center**: Current market price (horizontal divider)
**Left Column**: Buy limit orders (below current price)
**Right Column**: Sell limit orders (above current price)

**Order Entry Modes**:
1. **Mouse Trading Mode** - click price levels to place orders
2. **Order Entry Form** - traditional form with account/quantity selection
3. **Hotkeys** - keyboard shortcuts

**Visual Indicators**:
- Volume intensity shown at each price level
- Working orders displayed inline
- Bracket order visualization (Stop Loss + Take Profit pairs)

**Integration Features**:
- Can be combined with DOM Surface for full liquidity view
- Panel splitting to show multiple instruments
- Drag-and-drop order management

**Sources**:
- [Quantower DOM Surface Blog](https://www.quantower.com/blog/dom-surface-panel-for-deep-order-flow-analysis)
- [DOM Surface Help](https://help.quantower.com/quantower/analytics-panels/dom-surface)
- [DOM Trader Help](https://help.quantower.com/quantower/trading-panels/dom-trader)

---

## 3. Sierra Chart

### Chart DOM (Integrated Trading Interface)

Sierra Chart embeds the DOM **directly into chart windows** on the right side:

**Layout Structure**:

**Left Side**: Chart bars (candlesticks, bars, etc.)
**Right Side**: Chart DOM ladder overlay
- Buy column (left of price axis)
- Price ladder (center)
- Sell column (right of price axis)
- Market depth columns (Bid Size, Ask Size)

**Attachment Method**:
- Enable via `Trade >> Trading Chart DOM On`
- DOM appears as columns overlaying the chart's right edge
- Cannot be separated from chart (inherently integrated)
- Automatically enables `Chart Trade Mode On` and `Show Orders and Position`

**Column Customization**:
- Access via `Trade >> Customize Chart/Trade DOM Columns`
- Available columns:
  - Bid Size / Ask Size (market depth)
  - Buy / Sell order entry columns
  - Combined Bid/Ask Size (centered display)
  - Profit & Loss column (P&L at each price level)
  - Additional market data columns
- Use arrows to add/remove columns
- Move Up/Move Down buttons to reorder

**Order Entry Interactions**:
- **Left-click Buy/Sell columns** = Limit orders
- **Right-click Buy/Sell columns** = Stop orders
- **Drag order lines** horizontally to modify prices
- **Click quantity buttons** to adjust size
- **Click "X" buttons** to cancel

**Chart Integration**:
- Working orders shown as horizontal lines across chart
- Price ladder scales with chart zoom
- Auto-centering keeps current price visible
- Right-click price scale for scaling options

### Trade Window (Separate DOM)

Sierra Chart also offers a standalone **Trade Window** (traditional DOM):
- Can be attached to charts or remain independent
- Same column customization options as Chart DOM
- Suitable for multi-monitor setups

**Sources**:
- [Sierra Chart Trading and Chart DOM](https://www.sierrachart.com/index.php?page=doc/ChartTrading.html)
- [TicinoTrader Sierra DOM Setup](https://www.ticinotrader.ch/how-to-setup-and-customize-trading-dom-in-sierra-chart/)

---

## 4. Bookmap

### Heatmap + Order Book Integration

Bookmap is unique in that the **heatmap IS the primary visualization** (not a separate panel):

**Layout Structure**:

**Vertical Timeline** (center reference):
- **Right side of timeline**: Current Order Book (COB) in real-time
- **Left side of timeline**: Historical order book positions

**Visualization Elements**:

**Liquidity Heatmap**:
- Color intensity = order concentration
- Brighter areas = stronger supply/demand
- Example colors: orange line = large buy orders, yellow line = large sell orders
- Painted areas change dynamically as orders update

**Best Bid/Ask Lines**:
- **Green dotted line** = best bid (highest buy limit order)
- **Red dotted line** = best ask (lowest sell limit order)

**Trade Volume Bubbles**:
- **Green bubbles** = aggressive market buys (more buys than sells)
- **Red bubbles** = aggressive market sells (more sells than buys)
- **Mixed-color bubbles** = balanced buy/sell ratio
- Bubble size = execution volume

**Recent Transaction Indicator**:
- Rectangle on right side of screen shows most recent trade

**Performance**:
- Refreshes at **40 FPS** (frames per second)
- Ultra-high-speed tracking of market activity

**Trading Integration**:
- Analyze and trade directly from heatmap
- Click price levels to enter orders
- DOM overlay available for traditional ladder view

**Sources**:
- [Bookmap Heatmap Complete Guide](https://bookmap.com/blog/heatmap-in-trading-the-complete-guide-to-market-depth-visualization)
- [Bookmap Features](https://bookmap.com/en/features)
- [LuxAlgo Bookmap Insights](https://www.luxalgo.com/blog/bookmap-market-mapping-insights/)

---

## 5. Tiger Trade & CScalp

### Common Scalping Platform Patterns

Both platforms cater to crypto scalping with simplified interfaces:

**Tiger Trade**:

**DOM Display**:
- Full order book depth (all levels up and down)
- More extensive than CScalp's limited 1-2% range
- Right-click chart area → "Show DOM" to enable
- Dynamic DOM synced with chart price axis

**Chart Features**:
- Tick charts based on ticks, volume, delta, or range
- Drawing tools and indicators (advantage over CScalp)
- **Heatmap indicator** optimized for tick charts with value=1 (every bar = single aggregated trade)

**Additional Panels**:
- Trade feed (vertical list)
- Clusters for supply/demand analysis

**CScalp**:

**DOM Display**:
- Limited depth: only 1-2% above/below current price
- Faster and simpler than Tiger Trade
- Optimized for rapid order entry

**Advantages**:
- Simplified interface for quick decisions
- Minimal visual clutter
- Faster order execution (user reports)

**Comparison Summary**:
- **Tiger Trade** = more analytical tools, full DOM depth, drawing capabilities
- **CScalp** = faster execution, simpler DOM, limited depth

**Sources**:
- [Tiger Trade Chart Settings](https://support.tiger.com/english/windows/chart/chart-settings)
- [Bikotrading Tiger vs CScalp](https://bikotrading.com/programs-for-scalping-tigertrade-vs-cscalp-scalping-cryptocurrencies-on-apple-macos)
- [Tiger Trade Heatmap](https://tigertrade.freshdesk.com/en/support/solutions/articles/80001023416-heatmap)

---

## 6. Common UI/UX Patterns Across Platforms

### Universal Layout Convention: DOM-Centric Design

**The DOM Ladder is ALWAYS the center or primary anchor element.**

| Zone | Common Element | Purpose |
|------|----------------|---------|
| **Center** | DOM Ladder | Order entry, market depth, price reference |
| **Left Side** | Cluster/Footprint Chart OR Liquidity Heatmap | Historical order flow, volume analysis |
| **Right Side** | Volume Profile OR Trade Feed (Time & Sales) | Current trade activity, volume distribution |
| **Top** | Toolbar / Controls | Settings, instrument selector, timeframe |
| **Bottom** | Status Bar OR Tick Chart | Account info, connection status, horizontal trade visualization |

### Layout Variations by Platform Type

**Type A: Heatmap-First (Bookmap)**
- Heatmap fills entire chart area
- Current order book on right edge
- Historical liquidity extends left
- Bubble overlays for executions

**Type B: Cluster-First (ATAS)**
- Cluster chart (footprint) as main canvas
- DOM as sidebar or horizontal bars overlay
- Smart Tape as separate vertical panel
- Volume Profile as additional sidebar

**Type C: DOM-First (Quantower, Sierra Chart)**
- DOM ladder as primary interface
- Charts attach to the side or below
- Multiple DOMs for multi-instrument trading
- Modular panel system

**Type D: Simplified (CScalp, Tiger Trade)**
- Minimal DOM with limited depth
- Essential execution tools only
- Optimized for speed over analysis
- Mobile-friendly layouts

---

## 7. Tick Chart / Horizontal Trade Tape Implementation

### Concept

The "tick chart" in DOM contexts refers to a **horizontal trade visualization** where:
- **X-axis** = time progression (left = past, right = present)
- **Y-axis** = price (synced with DOM ladder)
- **Circles/dots** represent individual trades or aggregated tick clusters
- **Circle size** = trade volume (larger = bigger order)
- **Circle color** = buy (green) vs sell (red)

### ATAS Implementation: Tick Cluster

ATAS calls this the **"Tick Cluster"** chart:

**Format**:
- Horizontal bars/clusters (like candlesticks but tick-based)
- Each bar represents N trades (e.g., 144 ticks per bar)
- Calculation similar to Smart Tape but displayed as chart
- Evenly distributes trades across bars

**Aggregation Period Setting**:
- User configures tick count per bar (e.g., 100, 144, 500)
- Example: 144-tick chart = 1 bar per 144 executed trades
- Time per bar varies based on market activity

**Visual Elements**:
- Volume histogram overlays
- Delta coloring (net buy/sell pressure)
- Footprint data within each cluster

### Sierra Chart: Volume Dots Study

Sierra Chart offers a **Volume Dots** visualization:

**Features**:
- Circles plotted on price axis
- Size automatically adjusts relative to highest volume dot
- Both Total Volume and Dominant Side configurable
- Color intensity scales with volume

**Integration**:
- Overlays on regular charts
- Can be combined with Chart DOM
- Works with tick charts, time charts, volume charts

**Usage**:
- Identify where large volume occurred
- Spot institutional activity (large average trade size)
- Confirm trend strength (high volume at key levels)

### Quantower: DOM Surface Circles

Quantower's **DOM Surface** uses circles differently:

**Purpose**: Execution visualization (not tick aggregation)
- Red circles = sell order executions
- Green circles = buy order executions
- Size = individual order volume
- Plotted on heatmap at execution time/price

**Context**:
- Not a separate "tick chart" but integrated into heatmap
- Shows aggressive orders hitting the book
- Complements liquidity heatmap (passive orders)

### Common Patterns

| Feature | ATAS Tick Cluster | Sierra Volume Dots | Quantower Circles | Bookmap Bubbles |
|---------|-------------------|-------------------|-------------------|----------------|
| **X-Axis** | Time | Time | Time | Time |
| **Y-Axis** | Price | Price | Price | Price |
| **Shape** | Bars/clusters | Circles | Circles | Bubbles |
| **Size** | Volume | Volume | Order size | Aggression ratio |
| **Color** | Delta (buy/sell) | Dominant side | Buy (green) / Sell (red) | Buy / Sell / Mixed |
| **Aggregation** | Tick count per bar | None (per trade) | None (per execution) | None (per execution) |

### Aggregation Period Explained

**Definition**: Number of ticks (trades) required to form one bar/cluster.

**Examples**:
- **100-tick chart**: Every 100 trades = 1 bar
- **500-tick chart**: Every 500 trades = 1 bar
- **1-tick chart**: Every trade = 1 bar (maximum granularity)

**Time Variability**:
- In low-volatility periods, 100 ticks might take 5 minutes
- In high-volatility periods, 100 ticks might take 10 seconds
- Time-agnostic: focuses on activity, not clock time

**Visualization Options**:
- **Lines connecting dots**: Show price progression between trade clusters
- **Just dots**: Emphasize volume at discrete price levels
- **Bars with dots**: Combine OHLC bars with volume dots overlay

**Sources**:
- [ATAS Tick Charts Guide](https://atas.net/volume-analysis/tick-charts-in-simple-terms/)
- [Sierra Volume Dots Study](https://www.twofoxtrading.co.uk/docs/custom-study-instructions/volume-dots-study/)
- [ATAS Chart Periods and Types](https://help.atas.net/en/support/solutions/articles/72000602350-chart-periods-and-types)

---

## 8. User Configuration & Layout Customization

### Panel Splitting and Multi-Instrument Layouts

**Quantower**:
- Modular panel system with drag-and-drop
- Multi-link groups (sync multiple panels to same instrument)
- Save/load workspace templates
- Independent panel sizing

**Sierra Chart**:
- Chartbook system (tabbed chart windows)
- Each chart can have independent Chart DOM
- Trade Window can float or dock
- Custom column configurations per chart

**ATAS**:
- Workspace templates for different strategies
- Save panel arrangements with instruments
- Quick-switch between layouts
- Overlay vs sidebar DOM modes

**Bookmap**:
- Single-view focus (heatmap + order book integrated)
- Less modular but highly customized within main view
- Overlay trading tools on heatmap
- Multi-monitor support for separate instruments

### Common Configuration Options

| Setting | Purpose | Typical Values |
|---------|---------|----------------|
| **DOM Depth** | Number of price levels shown | 10, 20, 50, 100, Custom |
| **Tick Aggregation** | Trades per bar (tick charts) | 1, 100, 144, 500, 1000 |
| **Volume Threshold** | Minimum size to highlight | 10 lots, 100 lots, Custom |
| **Color Scheme** | Buy/Sell/Delta colors | Green/Red, Blue/Orange, Custom |
| **Font Size** | Readability | Small, Medium, Large |
| **Contrast** | Heatmap intensity | Low, Medium, High |
| **Auto-Center** | Follow price movement | On / Off |
| **Time Format** | Timestamp display | HH:MM:SS, Milliseconds |

---

## 9. Key Takeaways for Implementation

### Design Principles

1. **DOM is the anchor**: Always place DOM in center or as primary reference
2. **Price axis sync**: All panels (charts, heatmaps, tapes) must share Y-axis with DOM
3. **Left = historical context**: Cluster charts, footprints, heatmaps extend left from DOM
4. **Right = current activity**: Trade feeds, volume profiles, order book histograms go right
5. **Color consistency**: Green = buy/bid, Red = sell/ask across all visualizations
6. **Size = significance**: Larger circles/bars/bubbles = higher volume/importance

### Critical Features

**Must-Have**:
- Real-time DOM ladder with best bid/ask
- Order entry by clicking price levels
- Working order visualization (lines across chart)
- Volume/size indicators at each price level
- Color-coded buy/sell differentiation

**Should-Have**:
- Aggregated trade feed (like Smart Tape)
- Liquidity heatmap or historical DOM
- Volume dots/circles on charts
- Tick chart support with aggregation periods
- Customizable column layout

**Nice-to-Have**:
- Footprint/cluster chart modes (25+ variants like ATAS)
- Order book imbalance indicators
- Auto-centering on price movement
- Bracket order visualization
- Freeze/pause for analysis
- Historical playback mode

### Layout Recommendation

For a **Trading Container** panel combining DOM + cluster + tape:

```
+---------------------------------------------------------------+
|  Toolbar: [Instrument] [Timeframe] [Aggregation: 100 ticks]  |
+---------------------------------------------------------------+
|                    |              |                           |
|   Cluster Chart    |   DOM Ladder |    Time & Sales Tape     |
|   (Footprint)      |   (Center)   |    (Smart Tape style)    |
|                    |              |                           |
|  - Volume colors   | - Best B/A   |  - Time                   |
|  - Delta display   | - Order entry|  - Price                  |
|  - Price levels    | - Depth data |  - Volume                 |
|    synced with DOM |              |  - Buy/Sell color         |
|                    |              |                           |
+--------------------+--------------+---------------------------+
|  Volume Profile (horizontal histogram below)                 |
+---------------------------------------------------------------+
|  Status: [Position] [P&L] [Connection Status]                |
+---------------------------------------------------------------+
```

**Column Widths** (typical):
- Cluster Chart: 40-50% of width
- DOM Ladder: 20-30% of width
- Time & Sales: 30-40% of width

**Resizing**: User should be able to drag dividers between panels

### Data Synchronization

All components must share:
- **Price axis** (Y-axis alignment)
- **Current price line** (horizontal line across all panels)
- **Time reference** (for heatmaps and tick charts)
- **Instrument** (symbol/contract)
- **Best bid/ask updates** (real-time sync)

### Performance Targets

Based on professional platforms:
- **Update rate**: 40 FPS minimum (Bookmap standard)
- **Latency**: < 50ms from exchange data to screen
- **DOM depth**: Support 50-100 levels without lag
- **Tick data**: Handle 10,000+ ticks/second during high volume

---

## 10. Additional Resources

### Platform Documentation

- [ATAS Platform Overview](https://atas.net/)
- [Quantower Trading Platform](https://www.quantower.com/)
- [Sierra Chart Documentation](https://www.sierrachart.com/index.php?page=doc/DocMain.html)
- [Bookmap Features](https://bookmap.com/en/features)
- [Tiger Trade Help](https://support.tiger.com/english)

### Order Flow Education

- [ATAS Order Flow Analysis](https://www.quantvps.com/blog/atas-trading-platform-overview)
- [Top Order Flow Software](https://www.quantvps.com/blog/order-flow-trading-software)
- [DOM Trading Best Practices](https://www.quantvps.com/blog/dom-trading-ninjatrader-bookmap-quantower)
- [NinjaTrader Order Flow](https://ninjatrader.com/trading-platform/free-trading-charts/order-flow-trading/)

### Comparison Articles

- [Best Footprint Charting Software](https://coinpaper.com/12164/7-best-footprint-charting-software-for-traders-and-how-to-pick-one)
- [ATAS vs Tiger vs CScalp](https://bikotrading.com/atas-vs-tiger-vs-cscalp)
- [Order Flow Software Tools](https://optimusfutures.com/blog/order-flow-software/)

---

## Conclusion

Professional trading platforms universally adopt a **DOM-centric layout** with:
- **Center**: DOM ladder for order entry and market depth
- **Left**: Historical/analytical context (clusters, footprints, heatmaps)
- **Right**: Real-time activity (trade feeds, volume profiles)
- **Consistent Y-axis**: All panels synced to same price scale

The "tick chart" or "horizontal trade tape" visualizes individual trades as circles on a time/price grid, with size representing volume and color indicating buy/sell direction. Aggregation periods (e.g., "100 ticks per bar") group trades to reduce noise while preserving order flow insight.

Key differentiators between platforms:
- **ATAS**: Most comprehensive order flow tools (25+ cluster modes)
- **Quantower**: Best modular panel system (DOM Surface + DOM Trader)
- **Sierra Chart**: Deepest chart integration (Chart DOM overlay)
- **Bookmap**: Most innovative visualization (liquidity heatmap as primary interface)
- **CScalp/Tiger**: Fastest execution (simplified scalping-focused UI)

For implementation, prioritize: real-time DOM ladder, price-synced panels, aggregated trade feed, and customizable layout with drag-to-resize dividers.
