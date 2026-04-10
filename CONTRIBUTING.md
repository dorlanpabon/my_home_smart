# Contributing

Thanks for taking an interest in Tuya Desk Controller.

This project is still young, so the most valuable contributions are the ones that improve reliability, clarity, and real-world usefulness.

## Good contribution areas

- Tuya device compatibility improvements
- multi-gang detection edge cases
- UI polish that improves density without hurting clarity
- Windows packaging and release polish
- architecture improvements that prepare the project for `local_api`, `scheduler`, and `automation`
- tests based on real Tuya payload fixtures

## Before opening a change

1. Open an issue for larger changes or architecture work.
2. Keep changes focused and easy to review.
3. Prefer improving existing structure over introducing new framework-heavy abstractions.

## Development

Install dependencies:

```bash
npm install
```

Run the desktop app:

```bash
npm run tauri dev
```

Useful verification commands:

```bash
npm run test
npm run build
cd src-tauri && cargo test
```

## Code expectations

- Keep the frontend bound to normalized app models, not raw Tuya payloads.
- Keep Tuya Cloud auth, signing, and HTTP behavior inside the Rust backend.
- Add comments only where they add real value.
- Prefer small, direct changes over broad refactors without a clear payoff.

## Pull requests

Pull requests should ideally include:

- a short problem statement
- the intended user-facing outcome
- any technical tradeoffs
- notes on testing performed

## Reporting issues

Issues are especially helpful when they include:

- Tuya region and base URL
- whether the device is 1, 2, 3, or 4 gang
- the observed function/status codes when relevant
- whether the problem happens only in the desktop app or also after a manual refresh
