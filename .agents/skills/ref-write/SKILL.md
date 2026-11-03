---
name: ref-write
description: Writes or updates `.refs/` docs after a feature, fix, or lifecycle change — records only the non-obvious why, gotchas, deviations and gaps, verified against the code, in the terse style `ref-compact` enforces. Use every time a ref needs to be written or updated (the Operating Protocol's final step), or when asked to document something in the refs.
---

# Skill: Ref Write

## 🎯 Purpose
`.refs/` explains the *why* and the gotchas; code is the truth for the *what*
(see `.refs/conventions.md`). Every change that adds or alters behavior ends
with a ref update (Operating Protocol in `AGENTS.md`). This skill writes that
update so it is already compact — `ref-compact` should have nothing to cut.

## 📥 Arguments
Optional: the change to document (feature, fix, commit range) and/or the
target `.refs/` file. Default: the current branch's diff against `main` plus
any uncommitted changes.

## 🗂 Pre-Execution
1. Read the change: `git diff main...HEAD` and `git diff` (or the given
   range). Identify what is new, what moved, what was removed.
2. Read `.refs/conventions.md` and the `.refs/` file(s) that cover the touched
   subsystem in full. Use the `AGENTS.md` `.refs` index to pick the file.
3. Grep the refs for every symbol, file, flag or behavior the change touches —
   existing statements may now be stale.
4. If the change is user-visible (config, keybindings, CLI, OSC behavior),
   note the matching page under `docs/src/content/docs/` too.

## 📍 Where it goes
- Extend the file that owns the subsystem; put the fact in the most specific
  section, once.
- Create a new `.refs/<concern>.md` only for a genuinely new subsystem; add
  it to the `AGENTS.md` `.refs` index with a one-line scope description.
- Incidents and destructive-op lessons go in `.refs/incidents/`.

## ✅ What to write
- Why a non-obvious design exists, in one or two sentences.
- Gotchas and traps: what breaks if someone "simplifies" the code (platform
  quirks, ordering constraints, invariants two places must share).
- Deliberate deviations from a spec/handoff/standard and their reason.
- Known gaps and limitations the change leaves.
- Pointers to where things live (module → responsibility), briefly.

## ✂️ What not to write
- **Session narrative:** "previously…", "we tried…", "this fixes the bug
  where…", "now…", PR/issue history. State the lesson as a present-tense rule.
- **Restated code:** exact values, token tables, enumerations of items,
  step-by-step function walkthroughs. Point at the file instead.
- **Obvious things:** what the names or standard terminal/framework behavior
  already tell the reader.
- **Duplicates:** if the fact already lives elsewhere, link to it.
- **Speculation:** nothing that isn't backed by the code as it is now.

## 🔧 Keep the rest accurate
- Fix or remove any existing statement the change made stale (including in
  other `.refs/` files and `AGENTS.md` index descriptions).
- Name a symbol only after grepping that it exists in the current tree.

## 📝 Style
- Present tense, terse bullets, no filler intros.
- Bold the lead of a bullet only for a real trap.
- Match the surrounding file's headings and tone.

## 📤 Expected Output
- Edited (or new) `.refs/` files, plus the `docs/src/content/docs/` page when
  user-visible behavior changed.
- A short summary to the user: which files changed, what was added, and which
  stale statements were fixed.

## ⚠️ Rules and Constraints
- Docs-only: don't touch source code. If writing the ref reveals a code
  problem, report it instead of fixing it.
- No comments in code as a substitute — the ref is where the why goes.
- Don't commit unless asked; when asked, use the `commit` skill
  (`docs(refs): …`), separate from the code commit.
