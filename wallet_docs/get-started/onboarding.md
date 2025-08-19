# Onboarding

If an organization wants to participate in the NL-Wallet environment (i.e., you
have a development organization that is working on software the implements a
verifier or issuer which you want to test within the NL-Wallet ecosystem), you
can file a request to onboard.

Pending approval and successfully following the necessary technical steps, you
can then develop verifiers and issuers and test them with the pre-production
environment of the NL-Wallet platform.

For now, participation in the NL-Wallet environment is intended solely for
organizations that are developing software aiming to integrate with the
NL-Wallet eco-system.

This document details the onboarding steps to follow.

## Request

To apply for community membership which enables your organization to develop
integrations with the NL-Wallet platform, your organization [contacts the EDI
program][1] requesting participation in the NL-Wallet community.

The community manager of the EDI program will evaluate the request and after a
deliberation, organizes an intake in which the following subjects are discussed:

  * In what context does the applicant operate, and what are the ambitions
    and needs of the applicant organization;
  * Do we have enough resources to support the applicant (i.e., a large
    organization might require significantly more support);
  * Any relevant terms and conditions are discussed and agreed upon.

After the intake, there will be an internal (to the EDI program) evaluation of
the request. When approved, the process will continue (see next chapter). When
denied, the applicant will be informed and the process stops.

## Approval

If an applicant is approved, the community manager of the EDI program hands off
contact to the operations organization, which will plan and organize a kick-off
with the new participant (i.e., your organization's development team).

The kick-off session is an online session of about an hour where the participant
meets development and operations team members, and usually includes a quick
hack-a-thon to help the participant get started.

Relevant persons of participants' organization are added to our MS Teams support
channel and necessary steps for using the NL-Wallet platform are discussed, in
particular what the participant needs to get started (this documentation,
creation of CA and sharing public keys as described below).

## Certificates

The participants in the NL-Wallet ecosystem can generally be expressed as the
following three entities:

  * Users (people that use the Wallet app)
  * Issuers (entities that issue attested attributes)
  * Verifiers (entities that want to verify a users' attested attributes)

To develop integrations within the NL-Wallet ecosystem (i.e., a verifier, or an
issuer that can interact with the app), you need to have certain certificates
that ascertain the validity of a verifier and issuer, and that contain various
specific attributes that make the certicate work with our ecosystem.

These certificates are issued by a CA (Certificate Authority) and these CAs are
then trusted by the app, the issuers and the verifiers.

In the future, the NL-Wallet platform will likely facilitate or use a dedicated
CA entity that will be responsible for issuing certificates for both issuers and
verifiers, for both development and production purposes. This CA entity is not
yet available, and hence, we facilitate community participants by allowing them
to run their own CA.

In the next sub-chapters we will detail how you, as an approved participant of
the NL-Wallet platform, can create your own CA, and how you share the public key
of that CA with us.

### Create a CA

To create a Certificate Authority (CA) for development purposes, you can use the
`wallet_ca` CLI tool included in the NL-Wallet GitHub repository. We assume you
have followed the instructions in the `README.md` of the root of the NL-Wallet
repository, so you have rust installed, run on a supported platform, etc.

In the following code, we clone the nl-wallet repository, enter its directory,
set a target directory and specify an identifier (this identifier resembles your
organization, lowercase characters a-z, can end with numbers but not begin with
them). We then make sure the target directory exists, and invoke cargo (rust's
build tool) to in turn invoke `wallet_ca` which creates the CA certificate and
key.

Change at least the `IDENTIFIER` to match your organization name (should be
lowercase characters a-z, can end with numbers but may not begin with them).

```shell
git clone https://github.com/MinBZK/nl-wallet
cd nl-wallet
export TARGET_DIR=../ca-target
export IDENTIFIER=foocorp
mkdir -p "${TARGET_DIR}"
cargo run --manifest-path "wallet_core/Cargo.toml" --bin "wallet_ca" ca \
    --common-name "ca.${IDENTIFIER}.example.com" \
    --file-prefix "${TARGET_DIR}/ca.${IDENTIFIER}"
```

After executing the above commands, you will have two files in the target
directory you specified.

### Share CA Public Key with Operations Team

After you have created the CA certificate and its key, we need to share the
certificate with the operations team. the certificate will be called something
like this:

    ca.foocorp.crt.pem

Where "foocorp" is the name you previously used as identifier. You can e-mail
the public key to the e-mail address of the operations team. They will configure
this key in the issuer and reader trust anchors of the NL-Wallet. You will have
obtained the e-mail address to be used after approval, possibly at the kick-off.

Make sure you keep the key file safe, it should never be shared with any party
external to your organization (it's called something like `ca.foocorp.key.pem`).

## Resources

In this section we list various resources we have available for community
participants, and how you can make use of them.

### Github Issues (public)

You can use our [public issues][2] tracker when you encounter problems with our
software and/or documentation. We evaluate reported issues on tueday afternoon
and Friday mornings. This can then result in meeting topics for the online
consultation hour (see below).

### Teams Channel (private)

When you were approved, as mentioned before, a selection of people in your
organization was added to our MS Teams support channel(s). You can use these for
questions and/or feedback. Developers who work on the NL-Wallet, and operations
people who maintain the backend environment are also members of the/these
channel(s) and can answer various questions related to the NL-Wallet application
stack.

We have fixed times for when we check the messages in the channel: Tuesday
afternoon and Friday morning (same time we check for updates on our GitHub
issues).

### Online Consultation Hour (private, every Wednesday 10:00)

For things that are easier to discuss directly, we have a regular get-together
via MS Teams. This meeting only occurs when subjects for discussion are shared
and acknowledged through the Teams channel mentioned previously.

We try and arrange to make sure that the relevant experts are on the call (based
on the shared-in-advance subjects). When we do the consultation call, it will be
on a Wednesday morning from 10:00 to 11:00.

### Phone (only for service disruptions)

To keep things manageable from our side, we only prefer to use the previously
mentioned modes of contact. If however you encounter problems that you deem
serious (think service disruption, show-stopper bugs with the Wallet app or
backend), and it can't wait for one of the mentioned contact-moments, then it is
possible to call on the operations team, who will evaluate and work to find out
what's wrong. The operations team can, in turn, opt to engage the development
team (for example, when a software bug is the root cause of the issue).

The phone number will have been shared with you during onboarding approval.

### Sprint Review (private, every three weeks on Tuesday 10:00)

Every three weeks, the development team does a sprint review, where we present
what we have achieved in the sprint and what we plan to do in the next one.
Participating organizations can attend these sprint reviews to stay up-to-date
about developments. Requests for participation can be submitted via the EDI
community manager.

### Sharing Progress (demo)

A participant in the NL-Wallet community can request to share their progress
with the NL-Wallet team(s). If you as a participant feel you have something to
show and share during our three-weekly sprint review, please get in touch with
the EDI community manager so we can plan time during the sprint review.

## References

[1]: https://edi.pleio.nl/page/view/ddec7564-946d-49b1-b624-bee3900eb094/contact
[2]: https://github.com/minbzk/nl-wallet/issues
