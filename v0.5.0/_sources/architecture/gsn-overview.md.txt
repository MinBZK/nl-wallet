
# Core Assurance Goal for the EUDI Wallet

*“Relying Party (RP) establishes that the attribute (presented by Wallet) is a factually correct statement about the user at LoA High”* is the primary objective of the EUDI Wallet, as defined in the ARF. 

The ARF positions the wallet as an enabler of high-assurance attribute sharing. Accordingly, the RP’s justified confidence in the factual correctness of an attribute, rather than the mere secure transmission or presentation of data, is the decisive factor for trust at LoA High. 

The objective above is modeled using Goal Structuring Notation (GSN) to provide a structured, and auditable representation of the reasoning by which the Relying Party (RP) establishes its claim. 

The model decomposes this top-level goal into supporting sub-goals, assumptions, and solutions, demonstrating that the attribute has been correctly sourced, validated, and bound to the user identity.

The goal is decomposed as follows:

- <a href="../_static/img/gsn/rp_session.svg">rp_session_attribute</a>: Relying Party establishes that the attribute is a factually correct statement about the user at LoA High. (Main goal)

From the main goal, the following subgoals are identified:

<!-- use HTML link so that sphinx doesn't copy these images to _downloads/<hash>/filename.svg,
     as that breaks the internal links -->

- <a href="../_static/img/gsn/PID_issuer.gsn.svg">PID_issuer</a>: PID issuer issues at LoA High to wallets implementing Sole Control and ensures all issued attributes are factually correct during the lifetime of the attestations
- <a href="../_static/img/gsn/sole-control.gsn.svg">sole_control_main</a>: Wallet offers PID issuer assurance that User will have sole control over the PID.
- <a href="../_static/img/gsn/sole-control-2fa.gsn.svg">sole_control_2fa</a>: Wallet allows usage of PrivKey only after 2FA authentication of User
- <a href="../_static/img/gsn/sole-control-2fa-registration.gsn.svg">sole_control_2fa_registration</a>: Register possession & knowledge factors via enrollment message

The figure below shows how the subgoals relate:
<img src="../_static/img/gsn/architecture.svg" alt="Overview">


For convenience, an integral view is also provided:

<a href="../_static/img/gsn/complete.svg">Merged Diagram</a>: Main goal and subgoals from the sections above merged into this single diagram.