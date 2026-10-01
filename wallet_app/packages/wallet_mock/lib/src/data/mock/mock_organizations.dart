import 'package:wallet_core/core.dart';

import '../../util/extension/string_extension.dart';

final Map<String, Organization> kOrganizations = {
  kRvigId: _kRvigOrganization,
  kRdwId: _kRdwOrganization,
  kDuoId: _kDuoOrganization,
  kEmployerId: _kEmployerOrganization,
  kJusticeId: _kJustisOrganization,
  kMarketplaceId: _kMarketPlaceOrganization,
  kBarId: _kBarOrganization,
  kHealthInsuranceId: _kHealthInsurerOrganization,
  kHousingCorpId: _kHousingCorporationOrganization,
  kCarRentalId: _kCarRentalOrganization,
  kFirstAidId: _kFirstAidOrganization,
  kMunicipalityAmsterdamId: _kMunicipalityAmsterdamOrganization,
  kMunicipalityTheHagueId: _kMunicipalityTheHagueOrganization,
  kBankId: _kBankOrganization,
  kMonkeyBikeId: _kMonkeyBikeOrganization,
  kPharmacyId: _kPharmacyOrganization,
  kSupermarketId: _kSupermarket,
};

const kRvigId = 'rvig';
const kRdwId = 'rdw';
const kDuoId = 'duo';
const kEmployerId = 'employer_1';
const kJusticeId = 'justis';
const kMarketplaceId = 'marketplace';
const kBarId = 'bar';
const kHealthInsuranceId = 'health_insurer_1';
const kHousingCorpId = 'housing_corp_1';
const kCarRentalId = 'car_rental';
const kFirstAidId = 'first_aid';
const kMunicipalityAmsterdamId = 'municipality_amsterdam';
const kMunicipalityTheHagueId = 'municipality_the_hague';
const kBankId = 'bank';
const kMonkeyBikeId = 'monkey_bike';
const kPharmacyId = 'pharmacy';
const kSampleCityTheHague = 'Den Haag';
const kSupermarketId = 'supermarket';

const _kRvigOrganizationName = 'Rijksdienst voor Identiteits­gegevens';
final _kRvigOrganization = const Organization(
  //id: kRvigId,
  legalName: _kRvigOrganizationName,
  displayName: _kRvigOrganizationName,
  description: [
    ServiceDescription(
      translations: [
        LocalizedString(
          language: 'en',
          value: 'RvIG is the authority and director for the secure and reliable use of identity data.',
        ),
        LocalizedString(
          language: 'nl',
          value: 'RvIG is de autoriteit en regisseur van het veilig en betrouwbaar gebruik van identiteits­gegevens.',
        ),
      ],
    ),
  ],
  webUrl: 'https://www.rvig.nl/',
  privacyPolicyUrl: 'https://www.rvig.nl/over-deze-site/privacyverklaring-rijksdienst-voor-identiteitsgegevens',
  identifier: 'NTRNL-27373207',
  countryCode: 'NL',
);

final _kRdwOrganization = Organization(
  //id: kRdwId,
  legalName: 'Rijksdienst voor het Wegverkeer (RDW)',
  displayName: 'RDW',
  description: [
    ServiceDescription(
      translations:
          'De Rijksdienst voor het Wegverkeer (RDW) draagt bij aan een veilig, schoon, economisch en geordend wegverkeer.'
              .untranslated,
    ),
  ],
  identifier: 'NTRNL-27374436',
  countryCode: 'NL',
);

final _kDuoOrganization = Organization(
  //id: kDuoId,
  legalName: 'Dienst Uitvoering Onderwijs (DUO)',
  displayName: 'DUO',
  description: [
    ServiceDescription(
      translations:
          'Dienst Uitvoering Onderwijs (DUO) verzorgt onderwijs en ontwikkeling in opdracht van het Nederlandse ministerie van Onderwijs, Cultuur en Wetenschap.'
              .untranslated,
    ),
  ],
  identifier: 'NTRNL-50973029',
  countryCode: 'NL',
);

final _kEmployerOrganization = Organization(
  //id: kEmployerId,
  legalName: 'Werken voor Nederland',
  displayName: 'Werken voor Nederland',
  description: [
    ServiceDescription(
      translations:
          'Werken voor Nederland (onderdeel van De Rijksoverheid) is één van de grootste werkgevers van ons land. De kans dat jij jouw baan bij de Rijksoverheid vindt is dan ook behoorlijk groot.'
              .untranslated,
    ),
  ],
  identifier: 'NTRNL-98765431',
  countryCode: 'NL',
);

final _kJustisOrganization = Organization(
  //id: kJusticeId,
  legalName: 'Ministerie van Justitie en Veiligheid',
  displayName: 'Justis',
  description: [
    ServiceDescription(
      translations:
          'Screeningsautoriteit Justis beoordeelt de betrouwbaarheid van personen en organisaties ter bevordering van een veilige en rechtvaardige samenleving.'
              .untranslated,
    ),
  ],
  identifier: 'NTRNL-27378698',
  countryCode: 'NL',
);

const _kMarketPlaceOrganization = Organization(
  //id: kMarketplaceId,
  legalName: 'Marktplek B.V.',
  displayName: 'Marktplek',
  description: [
    ServiceDescription(
      translations: [
        LocalizedString(language: 'en', value: 'Easily sell your second-hand items online at Marktplek.'),
        LocalizedString(language: 'nl', value: 'Verkoop eenvoudig je tweedehands spullen via Marktplek.'),
      ],
    ),
  ],
  identifier: 'NTRNL-98765432',
  countryCode: 'NL',
  webUrl: 'https://www.marktplek.nl',
  privacyPolicyUrl: 'https://www.marktplek.nl/privacy',
);

final _kBarOrganization = Organization(
  //id: kBarId,
  legalName: 'Cafe de Dobbelaar',
  displayName: 'Cafe de Dobbelaar',
  description: [ServiceDescription(translations: 'Familiecafe sinds 1984.'.untranslated)],
  identifier: 'NTRNL-98765420',
  countryCode: 'NL',
);

final _kHealthInsurerOrganization = Organization(
  //id: kHealthInsuranceId,
  legalName: 'Zorgverzekeraar Z',
  displayName: 'Zorgverzekeraar Z',
  description: [
    ServiceDescription(
      translations:
          'Of het nu gaat om het regelen van zorg, het betalen van zorg of een gezond leven. Zorgverzekeraar Z zet zich elke dag in voor de gezondheid van haar klanten.'
              .untranslated,
    ),
  ],
  identifier: 'NTRNL-98765421',
  countryCode: 'NL',
);

final _kHousingCorporationOrganization = Organization(
  //id: kHousingCorpId,
  legalName: 'BeterWonen',
  displayName: 'BeterWonen',
  description: [
    ServiceDescription(
      translations: 'Moderne woningen voor iedereen in de Gemeente Den Haag en omstreken.'.untranslated,
    ),
  ],
  webUrl: 'https://beterwonen.nl',
  identifier: 'NTRNL-98765422',
  countryCode: 'NL',
);

final _kCarRentalOrganization = Organization(
  //id: kCarRentalId,
  legalName: 'CarRental',
  displayName: 'CarRental',
  description: [ServiceDescription(translations: 'Betrouwbaar huren.'.untranslated)],
  identifier: 'NTRNL-98765423',
  countryCode: 'NL',
);

final _kFirstAidOrganization = Organization(
  //id: 'first_aid',
  legalName: 'Healthcare Facility',
  displayName: 'Healthcare Facility',
  description: [
    ServiceDescription(
      translations:
          'Deze Healthcare Facility is fictief ter invulling van de Demo. Dit kan een zorginstelling zijn in Nederland of in het buitenland.'
              .untranslated,
    ),
  ],
  identifier: 'NTRNL-98765424',
  countryCode: 'NL',
);

const _kMunicipalityAmsterdamOrganization = Organization(
  //id: kMunicipalityAmsterdamId,
  legalName: 'Gemeente Amsterdam',
  displayName: 'Gemeente Amsterdam',
  description: [
    ServiceDescription(
      translations: [
        LocalizedString(language: 'en', value: 'Everything we do, we do for the city and the people of Amsterdam.'),
        LocalizedString(language: 'nl', value: 'Alles wat we doen, doen we voor de stad en de Amsterdammers.'),
      ],
    ),
  ],
  countryCode: 'NL',
  identifier: 'NTRNL-34366966',
  webUrl: 'https://www.amsterdam.nl',
  privacyPolicyUrl: 'https://www.amsterdam.nl/privacy',
);

final _kMunicipalityTheHagueOrganization = Organization(
  //id: kMunicipalityTheHagueId,
  legalName: "Gemeente 's-Gravenhage",
  displayName: 'Gemeente Den Haag',
  description: [
    ServiceDescription(
      translations:
          'Den Haag is een unieke stad waar we allemaal trots op zijn. Nieuwsgierig, divers en vol vertrouwen. Vrede en Recht.'
              .untranslated,
    ),
  ],
  identifier: 'NTRNL-27370927',
  countryCode: 'NL',
  webUrl: 'https://www.denhaag.nl',
);

const _kBankOrganization = Organization(
  //id: kBankId,
  legalName: 'XYZ Bank N.V.',
  displayName: 'XYZ Bank',
  description: [
    ServiceDescription(
      translations: [
        LocalizedString(language: 'en', value: 'The accessible bank for paying, saving and investing.'),
        LocalizedString(language: 'nl', value: 'Maak het leven makkelijk. Regel je financieën digitaal met Jouw Bank.'),
      ],
    ),
  ],
  identifier: 'NTRNL-12345678',
  countryCode: 'NL',
  webUrl: 'https://jouwbank.nl',
);

const _kMonkeyBikeOrganization = Organization(
  //id: kMonkeyBikeId,
  legalName: 'MonkeyBike Bezorgdiensten B.V.',
  displayName: 'MonkeyBike',
  description: [
    ServiceDescription(
      translations: [
        LocalizedString(language: 'en', value: 'Your groceries delivered to your home within 10 minutes.'),
        LocalizedString(
          language: 'nl',
          value: 'Razendsnel jouw boodschappen of bestelling bij jouw thuis. Altijd binnen 10 minuten.',
        ),
      ],
    ),
  ],
  countryCode: 'NL',
  webUrl: 'https://flitsbezorger-monkeybike.nl',
  identifier: 'NTRNL-3945-2932',
);

final _kPharmacyOrganization = Organization(
  //id: kPharmacyId,
  legalName: 'De Noord Apotheek',
  displayName: 'Apotheek',
  description: [ServiceDescription(translations: 'Al meer dan 25 jaar jouw betrouwbare apotheek.'.untranslated)],
  identifier: 'NTRNL-1234-1234',
  countryCode: 'NL',
  webUrl: 'https://denoordapotheek.nl',
);

final _kSupermarket = Organization(
  legalName: 'De Buurt Super',
  displayName: 'BuurtSuper',
  description: [ServiceDescription(translations: 'Al meer dan 25 jaar jouw betrouwbare supermarkt.'.untranslated)],
  identifier: 'NTRNL-1337-1337',
  countryCode: 'NL',
  webUrl: 'https://example.org',
);
