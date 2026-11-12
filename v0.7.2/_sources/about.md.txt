# About

NL Wallet is a secure app on your phone that lets you keep important personal information in one place, such as your name, age, or official documents like your ID card or driving licence.
With the wallet, you can easily prove who you are or show only the information a service needs.

For example:
- Showing you are over 18 without sharing your full date of birth.
- Confirming your identity when dealing with the government.
- Sharing a digital version of a diploma when applying for a job.

This is useful, because:
- You no longer need to upload scans of your passport.
- You share only what is needed, not your whole identity.
- Your documents are harder to fake, because they are digitally signed.
- It works across all EU countries.

You stay in control: you choose what to share, with whom, and when.

NL Wallet is intended for Dutch nationals. It offers convenience, stronger security, and protection against identity fraud — all while giving individuals greater control over their own information.

The app is being developed by Rijksoverheid (Dutch Government), in particular the Ministry of the Interior and Kingdom Relations (MinBZK) and is expected to be available to the public in 2027.

NL Wallet also makes things easier and safer for relying parties, which are organisations that need to check identity or personal information.

For these organisations, this means:
- More trust – the information comes directly from trusted sources, so it is harder to fake.
- Less sensitive data to store – no need to keep copies of passports or other documents.
- Easier compliance with privacy rules – only the necessary information is shared.
- Faster and smoother processes – users can identify themselves quickly, with fewer steps.
- European coverage – the same approach can be used in all EU countries.

This helps organisations to reduce fraud, protect personal data and offer a better experience to their users.


## NL Wallet within the EDI ecosystem

The NL Wallet is part of a broader ecosystem: the European Digital Identity (EUDI) Framework. This framework connects EU member states. Every member state also has their own national implementation of the framework. In the Netherlands this is called "EDI-Stelsel NL".

The EDI-stelsel defines the standards, roles and governance needed for issuance and use of identity-wallets, for both public and private organisations. 

Through this structure:
- Official issuers, like registry authorities or public agencies, provide verified identity- and other personal data, for instance from the national citizen registry. 
- Wallet holders (citizens or legal persons) store this data in NL Wallet.
- Service providers (public or private) can, with the user’s explicit consent, request certain data for a service (login, signature, proof of qualification, etc.).

The wallet acts as a universal, cross-sector tool: identification, authentication, data sharing, and digital signing, both within the Netherlands and across EU member states. 


## Guiding public values and how they are ensured
The design and development of NL Wallet and the EDI-stelsel are driven by a set of principal public values[^1] which have been translated into concrete safeguards and architectural choices.

[^1]: Sources: [Voortgangsrapportage Europese Digitale Identiteit](<https://www.tweedekamer.nl/kamerstukken/brieven_regering/detail?did=2022D32577&id=2022Z15589>) and [Bijlage 'Waarden, kansen en uitdagingen rond het Europese Digitale Identiteit raamwerk'](https://www.tweedekamer.nl/downloads/document?id=2022D32578) (PDF)

### User control over personal data (data sovereignty)
Users decide which personal information they store, when and with whom they share it. 

### Data minimisation and privacy
By default, only necessary data is exchanged. Wallets store data locally in a way that protects user privacy, avoiding central profiling or tracking. NL Wallet enables selective disclosure. This means, for instance, that you can share your birth date without sharing your name, or submitting only the data strictly needed for a service.

### Voluntary use and inclusivity
Use of NL Wallet is voluntary; existing login and identification methods remain available for those who prefer not to use it. This prevents exclusion of people who lack smartphone access or choose not to adopt a wallet. 

### Security
NL Wallet is built with safety measures to protect both the wallet and the data it contains. The system is designed to meet the requirements for the eIDAS “high” assurance level and will be independently assessed and certified against those requirements.

### Trust and legitimacy
Because the wallet is issued under a common framework (EUDI / eIDAS), it provides a standardised, interoperable trust base for both public and private services. Furthermore, the issued data originates from authentic, authoritative sources (like national registries), which results in data reliability and legal recognition.

### Transparency, open design and governance
The NL Wallet implementation is open source and the EDI-stelsel is being shaped with public-sector accountability. This means that there are defined roles for issuers, trust service providers, regulators and clear governance for what data can be issued, accepted, or revoked.

This documentation reflects the current implementation and will be updated with every software update. More information about the project can be found on [edi.pleio.nl](https://edi.pleio.nl)
