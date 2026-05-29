# Help content

The in-app **Help & information** screens are generated from the files in this folder. Every topic is two markdown files (English + Dutch) plus one entry in `help.yaml`. This README is for the person maintaining that content.

## File layout

```
help/
├── help.yaml   ← structure + titles/translations
├── en/<topicId>.md
└── nl/<topicId>.md
```

`topicId`s, file names, and YAML ids are lower-case `snake_case`.

## Add a topic

1. Pick a `topicId` — short, descriptive, e.g. `what_does_expired_mean`.
2. List it under the right group in `help.yaml`:

   ```yaml
   - subcategoryId: status
     topics:
       - groupId: help            # 'help' or 'information'
         topicIds:
           - what_does_expired_mean
   ```

3. Add the title in both languages under `translations:`:

   ```yaml
   translations:
     en:
       topics:
         what_does_expired_mean: What does "Expired" mean?
     nl:
       topics:
         what_does_expired_mean: Wat betekent "Verlopen"?
   ```

4. Create `en/what_does_expired_mean.md` and `nl/what_does_expired_mean.md` with the body (see formatting below).
5. **Validate** — from `wallet_app/`:

   ```
   fvm dart run tool/validate_help_content.dart
   ```

   Zero errors means it's ready to commit. The same validator runs in CI (`verify-help-content`).

## Edit / remove a topic

- **Edit text**: change the title in `help.yaml` `translations:` and/or the body in the two `.md` files.
- **Remove**: delete the YAML entry, the two `.md` files, and any `help://<topicId>` links pointing at it. Run the validator — it surfaces orphans and broken links.

## Markdown formatting

Blocks are separated by **a blank line**. Supported shapes:

| Shape | Looks like |
|---|---|
| Paragraph | plain prose, inline `**bold**` allowed |
| Subheading | a whole line of `**Bold text**` — nothing else |
| Bullet list | every line starts with `- ` (no numbered lists) |
| "Other questions" | one line of `[Label](help://topicId) \| [Label](help://other_topic)` |

### "Other questions" block

Place at the end of the topic. The app draws the heading and divider automatically — don't write a "See also:" label. The block must be on **one line**; line breaks disqualify the match. Targets the user has already visited on the current chain are filtered out.

## Cross-linking inside a paragraph

Use `[label](help://<topicId>)` anywhere in prose. The `topicId` must exist in `help.yaml` or the validator fails.

## Category icons

Each top-level category in `help.yaml` carries an `icon:` field. The icon name must be one of a curated list — the validator rejects anything else. **Adding a new icon requires a developer**; ask in #wallet-frontend (or open an issue) with the Material icon you need.

## Common pitfalls

- **Blank lines matter.** Without them, blocks merge into one paragraph.
- **File names are lower-case.** `en/What_Is_Wallet.md` won't match `topicId: what_is_wallet`.
- **Question subheadings end with `?`** — `**What does this mean?**`, not `**What does this mean**`. Descriptive subheadings (`**How it works**`) are fine without one.
- **No `1.` numbered lists** — use bullets.
