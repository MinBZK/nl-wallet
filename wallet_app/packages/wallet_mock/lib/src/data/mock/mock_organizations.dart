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

const _kRvigOrganizationName = 'Rijksdienst voor Identiteits­gegevens';
final _kRvigOrganization = Organization(
  //id: kRvigId,
  legalName: _kRvigOrganizationName.dutch,
  displayName: _kRvigOrganizationName.dutch,
  category: [
    const LocalizedString(language: 'en', value: 'Government'),
    const LocalizedString(language: 'nl', value: 'Overheid'),
  ],
  description: [
    const LocalizedString(
      language: 'en',
      value: 'RvIG is the authority and director for the secure and reliable use of identity data.',
    ),
    const LocalizedString(
      language: 'nl',
      value: 'RvIG is de autoriteit en regisseur van het veilig en betrouwbaar gebruik van identiteits­gegevens.',
    ),
  ],
  city: [
    const LocalizedString(language: 'en', value: 'The Hague, The Netherlands'),
    const LocalizedString(language: 'nl', value: 'Den Haag, Nederland'),
  ],
  image: const Image.asset(path: MockAssets.logo_rijksoverheid),
  webUrl: 'https://www.rvig.nl/',
  privacyPolicyUrl: 'https://www.rvig.nl/over-deze-site/privacyverklaring-rijksdienst-voor-identiteitsgegevens',
  kvk: '27373207',
);

final _kRdwOrganization = Organization(
  //id: kRdwId,
  legalName: 'Rijksdienst voor het Wegverkeer (RDW)'.untranslated,
  displayName: 'RDW'.untranslated,
  category: 'Overheid'.untranslated,
  description:
      'De Rijksdienst voor het Wegverkeer (RDW) draagt bij aan een veilig, schoon, economisch en geordend wegverkeer.'
          .untranslated,
  image: const Image.asset(path: MockAssets.logo_rdw),
);

final _kDuoOrganization = Organization(
  //id: kDuoId,
  legalName: 'Dienst Uitvoering Onderwijs (DUO)'.untranslated,
  displayName: 'DUO'.untranslated,
  category: 'Overheid'.untranslated,
  description:
      'Dienst Uitvoering Onderwijs (DUO) verzorgt onderwijs en ontwikkeling in opdracht van het Nederlandse ministerie van Onderwijs, Cultuur en Wetenschap.'
          .untranslated,
  image: const Image.asset(path: MockAssets.logo_rijksoverheid),
);

final _kEmployerOrganization = Organization(
  //id: kEmployerId,
  legalName: 'Werken voor Nederland'.untranslated,
  displayName: 'Werken voor Nederland'.untranslated,
  category: 'Bedrijf'.untranslated,
  description:
      'Werken voor Nederland (onderdeel van De Rijksoverheid) is één van de grootste werkgevers van ons land. De kans dat jij jouw baan bij de Rijksoverheid vindt is dan ook behoorlijk groot.'
          .untranslated,
  image: const Image.asset(path: MockAssets.logo_rijksoverheid),
);

final _kJustisOrganization = Organization(
  //id: kJusticeId,
  legalName: 'Ministerie van Justitie en Veiligheid'.untranslated,
  displayName: 'Justis'.untranslated,
  category: 'Overheid'.untranslated,
  description:
      'Screeningsautoriteit Justis beoordeelt de betrouwbaarheid van personen en organisaties ter bevordering van een veilige en rechtvaardige samenleving.'
          .untranslated,
  image: const Image.asset(path: MockAssets.logo_rijksoverheid),
);

final _kMarketPlaceOrganization = const Organization(
  //id: kMarketplaceId,
  legalName: [
    LocalizedString(language: 'en', value: 'Marktplek B.V.'),
    LocalizedString(language: 'nl', value: 'Marktplek B.V.'),
  ],
  displayName: [
    LocalizedString(language: 'en', value: 'Marktplek'),
    LocalizedString(language: 'nl', value: 'Marktplek'),
  ],
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
  kvk: '98765432',
  countryCode: 'nl',
  webUrl: 'https://www.marktplek.nl',
  privacyPolicyUrl: 'https://www.marktplek.nl/privacy',
);

final _kBarOrganization = Organization(
  //id: kBarId,
  legalName: 'Cafe de Dobbelaar'.untranslated,
  displayName: 'Cafe de Dobbelaar'.untranslated,
  category: 'Horeca'.untranslated,
  description: 'Familiecafe sinds 1984.'.untranslated,
  image: const Image.asset(path: MockAssets.logo_bar),
);

final _kHealthInsurerOrganization = Organization(
  //id: kHealthInsuranceId,
  legalName: 'Zorgverzekeraar Z'.untranslated,
  displayName: 'Zorgverzekeraar Z'.untranslated,
  category: 'Zorgverlener'.untranslated,
  description:
      'Of het nu gaat om het regelen van zorg, het betalen van zorg of een gezond leven. Zorgverzekeraar Z zet zich elke dag in voor de gezondheid van haar klanten.'
          .untranslated,
  image: const Image.asset(path: MockAssets.logo_zorgverzekeraar_z),
);

final _kHousingCorporationOrganization = Organization(
  //id: kHousingCorpId,
  legalName: 'BeterWonen'.untranslated,
  displayName: 'BeterWonen'.untranslated,
  category: 'Wooncorporatie'.untranslated,
  description: 'Moderne woningen voor iedereen in de Gemeente Den Haag en omstreken.'.untranslated,
  image: const Image.asset(path: MockAssets.logo_housing_corp),
  webUrl: 'https://beterwonen.nl',
  countryCode: 'nl',
  city: kSampleCityTheHague.untranslated,
  department: 'Secretariaat'.untranslated,
);

final _kCarRentalOrganization = Organization(
  //id: kCarRentalId,
  legalName: 'CarRental'.untranslated,
  displayName: 'CarRental'.untranslated,
  category: 'Autoverhuur'.untranslated,
  description: 'Betrouwbaar huren.'.untranslated,
  image: const Image.asset(path: MockAssets.logo_car_rental),
);

final _kFirstAidOrganization = Organization(
  //id: 'first_aid',
  legalName: 'Healthcare Facility'.untranslated,
  displayName: 'Healthcare Facility'.untranslated,
  category: 'Zorgverlener'.untranslated,
  description:
      'Deze Healthcare Facility is fictief ter invulling van de Demo. Dit kan een zorginstelling zijn in Nederland of in het buitenland.'
          .untranslated,
  image: const Image.asset(path: MockAssets.logo_first_aid),
);

final _kMunicipalityAmsterdamOrganization = Organization(
  //id: kMunicipalityAmsterdamId,
  legalName: [
    const LocalizedString(language: 'en', value: 'City of Amsterdam'),
    const LocalizedString(language: 'nl', value: 'Gemeente Amsterdam'),
  ],
  displayName: [
    const LocalizedString(language: 'en', value: 'City of Amsterdam'),
    const LocalizedString(language: 'nl', value: 'Gemeente Amsterdam'),
  ],
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
  countryCode: 'nl',
  kvk: '34366966',
  webUrl: 'https://www.amsterdam.nl',
  privacyPolicyUrl: 'https://www.amsterdam.nl/privacy',
);

final _kMunicipalityTheHagueOrganization = Organization(
  //id: kMunicipalityTheHagueId,
  legalName: 'Gemeente Den Haag'.untranslated,
  displayName: 'Gemeente Den Haag'.untranslated,
  category: 'Gemeente'.untranslated,
  description:
      'Den Haag is een unieke stad waar we allemaal trots op zijn. Nieuwsgierig, divers en vol vertrouwen. Vrede en Recht.'
          .untranslated,
  image: const Image.asset(path: MockAssets.logo_municipality_den_haag),
  department: 'Parkeren'.untranslated,
  city: kSampleCityTheHague.untranslated,
  countryCode: 'nl',
  webUrl: 'https://www.denhaag.nl',
);

const _kBankOrganization = Organization(
  //id: kBankId,
  legalName: [
    LocalizedString(language: 'en', value: 'XYZ Bank N.V.'),
    LocalizedString(language: 'nl', value: 'XYZ Bank N.V.'),
  ],
  displayName: [
    LocalizedString(language: 'en', value: 'XYZ Bank'),
    LocalizedString(language: 'nl', value: 'XYZ Bank'),
  ],
  category: [LocalizedString(language: 'en', value: 'Bank'), LocalizedString(language: 'nl', value: 'Bank')],
  description: [
    LocalizedString(language: 'en', value: 'The accessible bank for paying, saving and investing.'),
    LocalizedString(language: 'nl', value: 'Maak het leven makkelijk. Regel je financieën digitaal met Jouw Bank.'),
  ],
  image: Image.asset(path: MockAssets.logo_bank),
  department: [
    LocalizedString(language: 'en', value: 'Customer service'),
    LocalizedString(language: 'nl', value: 'Klantenservice'),
  ],
  countryCode: 'nl',
  city: [
    LocalizedString(language: 'en', value: 'Utrecht, The Netherlands'),
    LocalizedString(language: 'nl', value: 'Utrecht, Nederland'),
  ],
  webUrl: 'https://jouwbank.nl',
);

const _kMonkeyBikeOrganization = Organization(
  //id: kMonkeyBikeId,
  legalName: [
    LocalizedString(language: 'en', value: 'MonkeyBike Bezorgdiensten B.V.'),
    LocalizedString(language: 'nl', value: 'MonkeyBike Bezorgdiensten B.V.'),
  ],
  displayName: [
    LocalizedString(language: 'en', value: 'MonkeyBike'),
    LocalizedString(language: 'nl', value: 'MonkeyBike'),
  ],
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
  countryCode: 'nl',
  city: [
    LocalizedString(language: 'en', value: 'Groningen, The Netherlands'),
    LocalizedString(language: 'nl', value: 'Groningen, Nederland'),
  ],
  webUrl: 'https://flitsbezorger-monkeybike.nl',
  kvk: '3945-2932',
);

final _kPharmacyOrganization = Organization(
  //id: kPharmacyId,
  legalName: 'De Noord Apotheek'.untranslated,
  displayName: 'Apotheek'.untranslated,
  category: 'Apotheek'.untranslated,
  description: 'Al meer dan 25 jaar jouw betrouwbare apotheek.'.untranslated,
  image: const Image.asset(path: MockAssets.logo_zorgverzekeraar_z),
  kvk: '1234-1234',
  city: kSampleCityTheHague.untranslated,
  countryCode: 'nl',
  webUrl: 'https://denoordapotheek.nl',
);
