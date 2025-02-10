As part of our software delivery process, we do pull- or merge requests. These
are reviewed before being merged to the main branch and checked for compliance
with our [definition of done](./documentation/wow/definition-of-done.md).

This small merge-request guide describes succinctly how we should create an MR
and how we do a so-called MR review.

1. Decide if this is a standard (default) or basic MR. A standard MR
   is intended for regular work, always has a related issue ticket
   and is checked for various aspects documented in our definition
   of done. A basic MR is for quick fixes and/or forgotten things
   and does not always have a related issue ticket, and is checked
   only for unwanted content in commits and for technical debt TODO
   issues.

2. When you've chosen the MR type, a template text body will be
   inserted. Here you link the related issue ticket (if it's a
   standard MR) and insert a short description. Note that a full
   description, with other important information like implementation
   decisions and context for code changes should be part of the
   related issue ticket. A basic MR should not require extensive
   explanation.

3. In some cases, one or more of the specifically to-be-checkmarked
   items is not applicable to this MR. To assist the reviewer, you can
   use ~~strikethrough~~ on a list item below to indicate that that
   specific item is not relevant to this MR (i.e., don't delete the
   line, but use strikethrough instead).*

4. As the assignee of the MR, you implement the work in accordance
   with the definition of done; as a reviewer of the MR, you verify
   that such is the case.
