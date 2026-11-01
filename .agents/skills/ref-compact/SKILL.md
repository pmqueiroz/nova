---
name: ref-compact
description: Tightens the `.refs/` docs — strips agent-session narrative, drops content that only restates the code, fixes statements that drifted from the code, and keeps only the non-obvious why and gotchas. Use when the refs feel bloated, stale, or read like a conversation log, or when asked to compact/refine/tighten the refs.
---

# Skill: Ref Compact

## 🎯 Purpose
`.refs/` explains the *why* and the gotchas; code is the truth for the *what*
(see `.refs/conventions.md`). Over time agent sessions append narrative and
duplicate code into the refs. This skill brings them back to a short, accurate
reference.

## 📥 Arguments
Optional: one or more files under `.refs/` to limit the pass. Default: every
`.refs/*.md` (and `.refs/incidents/README.md`).

## 🗂 Pre-Execution
1. Read every target file in full.
2. List the module layout (`src/**`) so file/module claims can be checked.
3. Collect every statement that names a symbol, file, constant, flag or
   behavior that might have changed, and **verify it against the code**
   (grep for the symbol, read the relevant lines). Code wins every
   disagreement.
4. Also look for statements that contradict each other across files or
   sections — one side is stale.

## ✂️ What to cut
- **Session narrative:** "used to…", "the old … did X", "seen as…", "was
  reported…", "goes away when … lands", history of attempts, quotes of the
  design handoff's source code (JSX/CSS snippets), names of past sessions or
  conversations. Keep the lesson, rewritten in present tense as a rule or
  reason.
- **Restated code:** exact pixel/size/color values per component, token
  tables, enumerations of menu items or settings, step-by-step walkthroughs
  of a function. Point at the file instead.
- **Obvious things:** behavior any reader infers from the names or from
  standard terminal/framework behavior.
- **Repetition:** the same fact in several sections or files — keep it once,
  in the most specific place, and link if needed.
- **Status tables full of "done":** keep only gaps and deviations.
- **Generic boilerplate** not earned by this project (placeholder rules,
  template prose). Ask before removing if it looks intentional.

## ✅ What to keep
- Why a non-obvious design exists, in one or two sentences.
- Gotchas and traps: things that break if someone "simplifies" the code
  (platform quirks, ordering constraints, invariants two places must share).
- Deliberate deviations from a spec/handoff and their reason.
- Known gaps and limitations.
- Pointers to where things live (module → responsibility), briefly.
- External references (design project IDs, credits, licenses).

## 🔧 Fix drift
- Correct any statement the code contradicts; remove references to files or
  symbols that no longer exist.
- Mention every symbol name you keep only after grepping that it exists.
- Keep `AGENTS.md`'s `.refs` index descriptions accurate if a file's scope
  changes.

## 📝 Style
- Present tense, terse bullets, no filler intros.
- Bold the lead of a bullet only for a real trap.
- Don't add new content that isn't backed by the code.

## 📤 Expected Output
- Edited `.refs/` files, much shorter, with no stale claims.
- A summary to the user: what categories were cut, which stale statements
  were fixed (with the code evidence), and anything left in place that needs
  their decision.

## ⚠️ Rules and Constraints
- Don't touch source code or `docs/content/` — this skill is refs-only (flag
  user-doc drift instead of fixing it).
- Don't commit unless asked; when asked, use the `commit` skill
  (`docs(refs): …`).
