import 'package:wallet_core/core.dart';

import '../../util/extension/string_extension.dart';
import 'mock_assets.dart';

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
  category: [
    LocalizedString(language: 'en', value: 'Government'),
    LocalizedString(language: 'nl', value: 'Overheid'),
  ],
  description: [
    LocalizedString(
      language: 'en',
      value: 'RvIG is the authority and director for the secure and reliable use of identity data.',
    ),
    LocalizedString(
      language: 'nl',
      value: 'RvIG is de autoriteit en regisseur van het veilig en betrouwbaar gebruik van identiteits­gegevens.',
    ),
  ],
  city: [
    LocalizedString(language: 'en', value: 'The Hague, The Netherlands'),
    LocalizedString(language: 'nl', value: 'Den Haag, Nederland'),
  ],
  image: Image.asset(path: MockAssets.logo_rijksoverheid),
  webUrl: 'https://www.rvig.nl/',
  privacyPolicyUrl: 'https://www.rvig.nl/over-deze-site/privacyverklaring-rijksdienst-voor-identiteitsgegevens',
  identifier: 'NTRNL-27373207',
  countryCode: 'NL',
);

final _kRdwOrganization = Organization(
  //id: kRdwId,
  legalName: 'Rijksdienst voor het Wegverkeer (RDW)',
  displayName: 'RDW',
  category: 'Overheid'.untranslated,
  description:
      'De Rijksdienst voor het Wegverkeer (RDW) draagt bij aan een veilig, schoon, economisch en geordend wegverkeer.'
          .untranslated,
  image: const Image.asset(path: MockAssets.logo_rdw),
  identifier: 'NTRNL-27374436',
  countryCode: 'NL',
);

final _kDuoOrganization = Organization(
  //id: kDuoId,
  legalName: 'Dienst Uitvoering Onderwijs (DUO)',
  displayName: 'DUO',
  category: 'Overheid'.untranslated,
  description:
      'Dienst Uitvoering Onderwijs (DUO) verzorgt onderwijs en ontwikkeling in opdracht van het Nederlandse ministerie van Onderwijs, Cultuur en Wetenschap.'
          .untranslated,
  image: const Image.asset(path: MockAssets.logo_rijksoverheid),
  identifier: 'NTRNL-50973029',
  countryCode: 'NL',
);

final _kEmployerOrganization = Organization(
  //id: kEmployerId,
  legalName: 'Werken voor Nederland',
  displayName: 'Werken voor Nederland',
  category: 'Bedrijf'.untranslated,
  description:
      'Werken voor Nederland (onderdeel van De Rijksoverheid) is één van de grootste werkgevers van ons land. De kans dat jij jouw baan bij de Rijksoverheid vindt is dan ook behoorlijk groot.'
          .untranslated,
  image: const Image.asset(path: MockAssets.logo_rijksoverheid),
  identifier: 'NTRNL-98765431',
  countryCode: 'NL',
);

final _kJustisOrganization = Organization(
  //id: kJusticeId,
  legalName: 'Ministerie van Justitie en Veiligheid',
  displayName: 'Justis',
  category: 'Overheid'.untranslated,
  description:
      'Screeningsautoriteit Justis beoordeelt de betrouwbaarheid van personen en organisaties ter bevordering van een veilige en rechtvaardige samenleving.'
          .untranslated,
  image: const Image.asset(path: MockAssets.logo_rijksoverheid),
  identifier: 'NTRNL-27378698',
  countryCode: 'NL',
);

const _kMarketPlaceOrganization = Organization(
  //id: kMarketplaceId,
  legalName: 'Marktplek B.V.',
  displayName: 'Marktplek',
  category: [
    LocalizedString(language: 'en', value: 'Trading'),
    LocalizedString(language: 'nl', value: 'Winkelen'),
  ],
  description: [
    LocalizedString(language: 'en', value: 'Easily sell your second-hand items online at Marktplek.'),
    LocalizedString(language: 'nl', value: 'Verkoop eenvoudig je tweedehands spullen via Marktplek.'),
  ],
  image: Image.asset(path: MockAssets.logo_ecommerce),
  city: [
    LocalizedString(language: 'en', value: 'Zwolle, The Netherlands'),
    LocalizedString(language: 'nl', value: 'Zwolle, Nederland'),
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
  category: 'Horeca'.untranslated,
  description: 'Familiecafe sinds 1984.'.untranslated,
  image: const Image.asset(path: MockAssets.logo_bar),
  identifier: 'NTRNL-98765420',
  countryCode: 'NL',
);

final _kHealthInsurerOrganization = Organization(
  //id: kHealthInsuranceId,
  legalName: 'Zorgverzekeraar Z',
  displayName: 'Zorgverzekeraar Z',
  category: 'Zorgverlener'.untranslated,
  description:
      'Of het nu gaat om het regelen van zorg, het betalen van zorg of een gezond leven. Zorgverzekeraar Z zet zich elke dag in voor de gezondheid van haar klanten.'
          .untranslated,
  image: const Image.asset(path: MockAssets.logo_zorgverzekeraar_z),
  identifier: 'NTRNL-98765421',
  countryCode: 'NL',
);

final _kHousingCorporationOrganization = Organization(
  //id: kHousingCorpId,
  legalName: 'BeterWonen',
  displayName: 'BeterWonen',
  category: 'Wooncorporatie'.untranslated,
  description: 'Moderne woningen voor iedereen in de Gemeente Den Haag en omstreken.'.untranslated,
  image: const Image.asset(path: MockAssets.logo_housing_corp),
  webUrl: 'https://beterwonen.nl',
  identifier: 'NTRNL-98765422',
  countryCode: 'NL',
  city: kSampleCityTheHague.untranslated,
  department: 'Secretariaat'.untranslated,
);

final _kCarRentalOrganization = Organization(
  //id: kCarRentalId,
  legalName: 'CarRental',
  displayName: 'CarRental',
  category: 'Autoverhuur'.untranslated,
  description: 'Betrouwbaar huren.'.untranslated,
  image: const Image.asset(path: MockAssets.logo_car_rental),
  identifier: 'NTRNL-98765423',
  countryCode: 'NL',
);

final _kFirstAidOrganization = Organization(
  //id: 'first_aid',
  legalName: 'Healthcare Facility',
  displayName: 'Healthcare Facility',
  category: 'Zorgverlener'.untranslated,
  description:
      'Deze Healthcare Facility is fictief ter invulling van de Demo. Dit kan een zorginstelling zijn in Nederland of in het buitenland.'
          .untranslated,
  image: const Image.asset(path: MockAssets.logo_first_aid),
  identifier: 'NTRNL-98765424',
  countryCode: 'NL',
);

final _kMunicipalityAmsterdamOrganization = Organization(
  //id: kMunicipalityAmsterdamId,
  legalName: 'Gemeente Amsterdam',
  displayName: 'Gemeente Amsterdam',
  category: [
    const LocalizedString(language: 'en', value: 'Municipality'),
    const LocalizedString(language: 'nl', value: 'Gemeente'),
  ],
  description: [
    const LocalizedString(language: 'en', value: 'Everything we do, we do for the city and the people of Amsterdam.'),
    const LocalizedString(language: 'nl', value: 'Alles wat we doen, doen we voor de stad en de Amsterdammers.'),
  ],
  image: const Image.asset(path: MockAssets.logo_municipality_amsterdam),
  city: 'Amsterdam'.untranslated,
  countryCode: 'NL',
  identifier: 'NTRNL-34366966',
  webUrl: 'https://www.amsterdam.nl',
  privacyPolicyUrl: 'https://www.amsterdam.nl/privacy',
);

final _kMunicipalityTheHagueOrganization = Organization(
  //id: kMunicipalityTheHagueId,
  legalName: "Gemeente 's-Gravenhage",
  displayName: 'Gemeente Den Haag',
  category: 'Gemeente'.untranslated,
  description:
      'Den Haag is een unieke stad waar we allemaal trots op zijn. Nieuwsgierig, divers en vol vertrouwen. Vrede en Recht.'
          .untranslated,
  image: const Image.asset(path: MockAssets.logo_municipality_den_haag),
  identifier: 'NTRNL-27370927',
  department: 'Parkeren'.untranslated,
  city: kSampleCityTheHague.untranslated,
  countryCode: 'NL',
  webUrl: 'https://www.denhaag.nl',
);

const _kBankOrganization = Organization(
  //id: kBankId,
  legalName: 'XYZ Bank N.V.',
  displayName: 'XYZ Bank',
  category: [
    LocalizedString(language: 'en', value: 'Bank'),
    LocalizedString(language: 'nl', value: 'Bank'),
  ],
  description: [
    LocalizedString(language: 'en', value: 'The accessible bank for paying, saving and investing.'),
    LocalizedString(language: 'nl', value: 'Maak het leven makkelijk. Regel je financieën digitaal met Jouw Bank.'),
  ],
  image: Image.asset(path: MockAssets.logo_bank),
  identifier: 'NTRNL-12345678',
  department: [
    LocalizedString(language: 'en', value: 'Customer service'),
    LocalizedString(language: 'nl', value: 'Klantenservice'),
  ],
  countryCode: 'NL',
  city: [
    LocalizedString(language: 'en', value: 'Utrecht, The Netherlands'),
    LocalizedString(language: 'nl', value: 'Utrecht, Nederland'),
  ],
  webUrl: 'https://jouwbank.nl',
);

const _kMonkeyBikeOrganization = Organization(
  //id: kMonkeyBikeId,
  legalName: 'MonkeyBike Bezorgdiensten B.V.',
  displayName: 'MonkeyBike',
  category: [
    LocalizedString(language: 'en', value: 'Delivery service'),
    LocalizedString(language: 'nl', value: 'Bezorgdienst'),
  ],
  description: [
    LocalizedString(language: 'en', value: 'Your groceries delivered to your home within 10 minutes.'),
    LocalizedString(
      language: 'nl',
      value: 'Razendsnel jouw boodschappen of bestelling bij jouw thuis. Altijd binnen 10 minuten.',
    ),
  ],
  image: Image.asset(path: MockAssets.logo_monkeybike),
  department: [
    LocalizedString(language: 'en', value: 'Online marketing'),
    LocalizedString(language: 'nl', value: 'Online marketing'),
  ],
  countryCode: 'NL',
  city: [
    LocalizedString(language: 'en', value: 'Groningen, The Netherlands'),
    LocalizedString(language: 'nl', value: 'Groningen, Nederland'),
  ],
  webUrl: 'https://flitsbezorger-monkeybike.nl',
  identifier: 'NTRNL-3945-2932',
);

final _kPharmacyOrganization = Organization(
  //id: kPharmacyId,
  legalName: 'De Noord Apotheek',
  displayName: 'Apotheek',
  category: 'Apotheek'.untranslated,
  description: 'Al meer dan 25 jaar jouw betrouwbare apotheek.'.untranslated,
  image: const Image.asset(path: MockAssets.logo_zorgverzekeraar_z),
  identifier: 'NTRNL-1234-1234',
  city: kSampleCityTheHague.untranslated,
  countryCode: 'NL',
  webUrl: 'https://denoordapotheek.nl',
);

final _kSupermarket = Organization(
  legalName: 'De Buurt Super',
  displayName: 'BuurtSuper',
  category: 'Supermarkt'.untranslated,
  description: 'Al meer dan 25 jaar jouw betrouwbare supermarkt.'.untranslated,
  image: const Image.asset(path: MockAssets.logo_ecommerce),
  identifier: 'NTRNL-1337-1337',
  city: kSampleCityTheHague.untranslated,
  countryCode: 'NL',
  webUrl: 'https://example.org',
);
