# macOS Finder Icon Intercept & Redirect — Research

## Goal
Detect clicks on the Finder icon in the macOS Dock and redirect them to open an alternative file manager (e.g., Bloom) instead. Eventually support a UI to configure which apps run.

---

## How Others Already Do This

Several open-source macOS utilities intercept Dock icon clicks. The key technical approaches are:

### 1. CGEvent Tap — Global Mouse Click Interception
All solutions use `CGEvent.tapCreate(tap: .cghidEventTap, ...)` to install a global event tap that intercepts `leftMouseDown` events before the Dock processes them.

**Required macOS Permissions:**
- **Accessibility** (to inspect Dock UI elements)
- **Input Monitoring** (to create the HID event tap)

**Key repos demonstrating this pattern:**
- [`dockmint`](https://github.com/apotenza92/dockmint) — "macOS app to customize Dock icon click, double-click, and scroll actions"
- [`Click2Minimize`](https://github.com/hatimhtm/Click2Minimize) — "Swift + CGEvent tap + Accessibility"
- [`DockClickMinimize`](https://github.com/anthonycbl/DockClickMinimize) — "intercepts Dock icon clicks"
- [`Click2Hide-Stable`](https://github.com/rpranjan11/Click2Hide-Stable) — optimized for macOS Sequoia

---

## Technical Architecture

### High-Level Flow

```
1. User clicks Finder icon in Dock
2. Our CGEvent tap catches the leftMouseDown event
3. We check if the click is within the Dock's bounds
4. We identify which Dock item was clicked
5. If it's Finder → consume the event (don't pass to Dock)
6. Launch the configured alternative app (e.g., Bloom)
7. If it's not Finder → pass the event through normally
```

### Core Implementation Pieces

#### A. Event Tap Setup
```swift
let eventMask: CGEventMask = (1 << CGEventType.leftMouseDown.rawValue)

guard let eventTap = CGEvent.tapCreate(
    tap: .cghidEventTap,
    place: .headInsertEventTap,  // or .tailAppendEventTap
    options: .defaultTap,
    eventsOfInterest: eventMask,
    callback: { proxy, type, event, refcon in
        let appDelegate = Unmanaged<AppDelegate>.fromOpaque(refcon!).takeUnretainedValue()
        return AppDelegate.handleDockClick(proxy: proxy, type: type, event: event, delegate: appDelegate)
    },
    userInfo: Unmanaged.passUnretained(self).toOpaque()
) else {
    // Accessibility permission missing
    return
}

let runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, eventTap, 0)
CFRunLoopAddSource(CFRunLoopGetCurrent(), runLoopSource, .commonModes)
```

#### B. Dock Item Detection (Two Approaches)

**Approach 1: AppleScript (Click2Minimize style)**
```swift
let script = """
tell application "System Events"
    set dockItemList to {}
    tell process "Dock"
        set dockItems to every UI element of list 1
        repeat with dockItem in dockItems
            set dockPosition to position of dockItem
            set dockSize to size of dockItem
            set appID to name of dockItem
            set end of dockItemList to {dockPosition, dockSize, appID}
        end repeat
        return dockItemList
    end tell
end tell
"""
```
- Pros: Simple, works
- Cons: Slower, AppleScript can be flaky, requires UI scripting permission

**Approach 2: Accessibility API (Dockmint style — Recommended)**
```swift
import ApplicationServices

// Get the element at the mouse position
let system = AXUIElementCreateSystemWide()
var element: AXUIElement?
AXUIElementCopyElementAtPosition(system, Float(point.x), Float(point.y), &element)

// Check if it's in the Dock process
var pid: pid_t = 0
AXUIElementGetPid(element!, &pid)
let app = NSRunningApplication(processIdentifier: pid)
let isDock = app?.bundleIdentifier == "com.apple.dock"

// Get the subrole to identify what type of item
var subrole: AnyObject?
AXUIElementCopyAttributeValue(element!, kAXSubroleAttribute as CFString, &subrole)

// For app icons, subrole == "AXApplicationDockItem"
// For folders, subrole == "AXFolderDockItem"

// Get the URL to identify the app
var urlValue: AnyObject?
AXUIElementCopyAttributeValue(element!, kAXURLAttribute as CFString, &urlValue)
if let url = urlValue as? URL,
   let bundle = Bundle(url: url),
   let bundleId = bundle.bundleIdentifier {
    // bundleId will be "com.apple.finder" for Finder
}
```
- Pros: Fast, reliable, no AppleScript needed
- Cons: More code, still requires Accessibility permission

#### C. Identifying the Finder Icon
The Finder bundle identifier is: **`com.apple.finder`**

When the Dock hit-test resolves to this bundle ID, we know the user clicked Finder.

#### D. Launching the Alternative App
```swift
// Option 1: Launch by bundle ID
let bloomURL = NSWorkspace.shared.urlForApplication(withBundleIdentifier: "com.yourcompany.bloom")
if let url = bloomURL {
    NSWorkspace.shared.openApplication(at: url, configuration: .init())
}

// Option 2: Launch by URL (for folders/files)
let fileURL = URL(fileURLWithPath: "/Users/alex/Desktop")
NSWorkspace.shared.open(fileURL, withApplicationAt: bloomURL, configuration: .init())
```

#### E. Consuming the Event
To prevent the Dock from processing the click, return `nil` from the event tap callback:
```swift
return nil  // Event consumed, Dock never sees it
```

To pass the event through normally:
```swift
return Unmanaged.passUnretained(event)  // Event passes through
```

---

## Hiding the Finder Icon from the Dock

This is harder. Finder is a "persistent" Dock item that macOS always shows. There are two approaches:

### Option 1: Modify DockMenus.plist (Requires Disabling SIP)
1. Disable System Integrity Protection (SIP)
2. Modify `/System/Library/CoreServices/Dock.app/Contents/Resources/DockMenus.plist`
3. Add `REMOVE_FROM_DOCK` command for `finder-running`
4. Re-enable SIP
5. Use a LaunchAgent to auto-remove Finder on boot

**Resources:**
- [`hide-finder-trash-dock-icons`](https://github.com/jesscxc/hide-finder-trash-dock-icons)
- [`hide-macos-dock-items`](https://github.com/echocrow/hide-macos-dock-items)

### Option 2: Don't Hide It — Just Intercept It
Much simpler: Leave Finder in the Dock but intercept all its clicks. The user will see the Finder icon, but clicking it opens their preferred app instead.

This is the recommended approach for v1.

---

## Recommended Tech Stack

Based on the open-source implementations and your goal:

| Component | Technology | Why |
|-----------|------------|-----|
| Core app | Swift + SwiftUI | Native macOS, fast, direct access to CGEvent/Accessibility APIs |
| Event interception | `CGEvent.tapCreate` + `CFMachPort` | Proven pattern used by all Dock intercept utilities |
| Dock detection | `AXUIElementCopyElementAtPosition` | More reliable than AppleScript |
| App launch | `NSWorkspace.shared.openApplication` | Native API for launching apps |
| UI for configuration | SwiftUI | Easy settings panel for choosing which app to open |
| Launch at login | `SMAppService` | Modern macOS API for login items (macOS 13+) |

---

## Known Apps That Already Do This

| App | What It Does | Open Source? |
|-----|-------------|--------------|
| **Dockmint** | Customizes Dock click/scroll actions | ✅ Yes |
| **Click2Minimize** | Click dock icon to minimize windows | ✅ Yes |
| **DockClickMinimize** | Intercepts Dock icon clicks | ✅ Yes |
| **Click2Hide-Stable** | Click-to-hide, Sequoia-optimized | ✅ Yes |
| **Path Finder** | Full Finder replacement | ❌ No (commercial) |
| **ForkLift** | Finder alternative | ❌ No (commercial) |
| **Default Folder X** | Enhances Finder open/save dialogs | ❌ No (commercial) |

---

## Implementation Plan

### Phase 1: Basic Interceptor (MVP)
1. Create SwiftUI menu bar app (`LSUIElement` / `NSApplication.setActivationPolicy(.accessory)`)
2. Request Accessibility permission on first launch
3. Install `CGEvent.tapCreate` for `leftMouseDown`
4. Use `AXUIElementCopyElementAtPosition` to detect if click is on Finder
5. If Finder → consume event and launch Bloom (hardcoded)
6. Otherwise → pass event through

### Phase 2: Configuration UI
1. Add SwiftUI settings window
2. Let user select which app opens when Finder is clicked
3. Store preference in `UserDefaults`
4. Support per-app overrides (e.g., different app for different scenarios)

### Phase 3: Polish
1. Launch at login support (`SMAppService`)
2. Handle edge cases (Dock magnification, different Dock positions, multiple monitors)
3. Auto-updater

---

## Key Code References

- **Click2Minimize** `AppDelegate.swift`: https://github.com/hatimhtm/Click2Minimize/blob/main/Click2Minimize/AppDelegate.swift
- **Dockmint** `DockClickEventTap.swift`: https://github.com/apotenza92/dockmint/blob/main/Dockmint/DockClickEventTap.swift
- **Dockmint** `DockHitTest.swift`: https://github.com/apotenza92/dockmint/blob/main/Dockmint/DockHitTest.swift
- **Dockmint** `DockDecisionEngine.swift`: https://github.com/apotenza92/dockmint/blob/main/Dockmint/DockDecisionEngine.swift

---

## Risks & Considerations

| Risk | Mitigation |
|------|------------|
| Accessibility permission required | Onboarding flow that guides user to System Settings |
| Input Monitoring permission required | Same as above |
| Event tap disabled by timeout | Re-enable in callback (see Dockmint's `recoverAfterTapTimeout()`) |
| Event tap disabled by user input | Re-enable in callback |
| Dock position changes (left/bottom/right) | Query `CGDisplayBounds` and check all edges |
| Dock magnification affects hit area | Use `neutralBackgroundPoint` approach for fallback |
| Multi-monitor setups | Use `CGGetActiveDisplayList` to find correct display bounds |
| Finder is not running | `NSRunningApplication` may not find it; handle gracefully |
| Future macOS changes | Keep event tap logic minimal; rely on Accessibility APIs |
| macOS security warnings | Code signing + notarization required for distribution |

---

## Next Steps

1. **Scaffold a SwiftUI menu bar app** with Accessibility permission onboarding
2. **Implement the event tap** and test basic click interception
3. **Implement Dock hit-testing** using `AXUIElementCopyElementAtPosition`
4. **Test Finder detection** — verify bundle ID `com.apple.finder` is correctly identified
5. **Implement app launching** — open Bloom when Finder is clicked
6. **Add settings UI** — let user pick which app to open

---

*Research compiled from open-source Dock interception utilities. All referenced repos are publicly available on GitHub.*
