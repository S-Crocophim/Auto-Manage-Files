# Design Direction: Auto File Organizer

## Identity & Purpose
Auto File Organizer is a local-first, offline desktop utility designed for managing and organizing files automatically with precision, safety, and transparency. The visual language conveys reliability, clarity, and utilitarian efficiency — akin to a well-crafted operating system utility rather than a flashy consumer SaaS landing page.

## Dials
- **ENERGY**: 1 (Calm, focused, restrained, tool-first)
- **RHYTHM**: 1 (Predictable, clean hierarchy, structured panels)
- **MOTION**: 1 (Functional hover and focus transitions only, immediate state changes, full reduced-motion respect)

## Color System
A restrained, high-contrast palette limited to neutral foundations, one core brand accent, and essential semantic status colors:

- **Surface & Backgrounds**:
  - Light mode: Pure white surfaces (`#ffffff`), soft slate backgrounds (`#f8fafc`), defined borders (`#e2e8f0`).
  - Dark mode: Deep charcoal surfaces (`#181a20`), midnight backgrounds (`#0f1115`), subtle borders (`#262a36`).
- **Brand Primary**:
  - Light: `#4f46e5` (Indigo-600, contrast 5.9:1 against white).
  - Dark: `#818cf8` (Indigo-400, contrast 7.8:1 against `#181a20`).
- **Semantic Accents**:
  - Success: Emerald (`#059669` light / `#34d399` dark) for verified transfers and active monitoring.
  - Warning: Amber (`#b45309` light / `#fbbf24` dark) for pending actions and conflicts requiring review.
  - Danger: Rose (`#e11d48` light / `#fb7185` dark) for failed operations, deletes, and unmonitored errors.

## Typography
- **Font Stack**: System sans-serif stack (`system-ui, -apple-system, "Segoe UI Variable", "Segoe UI", Roboto, sans-serif`) for native operating system integration without web font downloads.
- **Hierarchy**: Clear, honest headings without oversized tracking or decorative uppercase eyebrow badges.
- **Readability**: Minimum 4.5:1 WCAG AA contrast on all normal text, including metadata and labels.

## Spacing & Shape Tokens
- **Border Radius**:
  - Small (`6px`): Status badges, tags, and small controls.
  - Medium (`10px`): Interactive buttons, text inputs, and table containers.
  - Large (`14px`): Structural cards, panels, and modal dialogs.
- **Elevation**: Flat surfaces with 1px border definition. Elevation is reserved strictly for floating overlays (modals, dropdowns, toast alerts).

## One-Line Decision Justifications (R-31)
- **Why system sans-serif typography?** It integrates naturally with Windows desktop conventions and guarantees zero network latency.
- **Why an indigo brand accent?** Indigo provides high visual contrast against light/dark neutrals without aggressive saturation.
- **Why structured panel layouts?** Desktop utilities require clear spatial organization of rules, filters, and audit logs.
- **Why flat borders instead of drop shadows?** Flat geometry keeps complex multi-column data readable and calm.
- **Why full light and dark mode adaptation?** Users working in daylight or low light need comfortable, compliant contrast in both themes.
- **Why native action icons over decorative sparkles?** Icons must directly communicate file operations (organize, folder, audit, settings).
