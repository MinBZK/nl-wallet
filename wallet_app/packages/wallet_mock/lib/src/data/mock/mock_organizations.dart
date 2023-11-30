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
const kMunicipalityTheHagueId = 'municipality_the_hague';
const kBankId = 'bank';
const kMonkeyBikeId = 'monkey_bike';
const kPharmacyId = 'pharmacy';

final _kRvigOrganization = Organization(
  //id: kRvigId,
  legalName: 'Rijksdienst voor Identiteitsgegevens'.untranslated,
  displayName: 'Rijksdienst voor Identiteitsgegevens'.untranslated,
  //type: 'Overheid'.untranslated,
  description:
      'Rijksdienst voor Identiteitsgegevens is de autoriteit en regisseur van het veilig en betrouwbaar gebruik van identiteitsgegevens.'
          .untranslated,
  image: Image.asset(path: MockAssets.logo_rijksoverheid),
);

final _kRdwOrganization = Organization(
  //id: kRdwId,
  legalName: 'Rijksdienst voor het Wegverkeer (RDW)'.untranslated,
  displayName: 'RDW'.untranslated,
  //type: 'Overheid'.untranslated,
  image: Image.asset(path: MockAssets.logo_rdw),
  description:
      'De Rijksdienst voor het Wegverkeer (RDW) draagt bij aan een veilig, schoon, economisch en geordend wegverkeer.'
          .untranslated,
);

final _kDuoOrganization = Organization(
  //id: kDuoId,
  legalName: 'Dienst Uitvoering Onderwijs (DUO)'.untranslated,
  displayName: 'DUO'.untranslated,
  //type: 'Overheid'.untranslated,
  image: Image.asset(path: MockAssets.logo_rijksoverheid),
  description:
      'Dienst Uitvoering Onderwijs (DUO) verzorgt onderwijs en ontwikkeling in opdracht van het Nederlandse ministerie van Onderwijs, Cultuur en Wetenschap.'
          .untranslated,
);

final _kEmployerOrganization = Organization(
  //id: kEmployerId,
  legalName: 'Werken voor Nederland'.untranslated,
  displayName: 'Werken voor Nederland'.untranslated,
  //type: 'Bedrijf'.untranslated,
  description:
      'Werken voor Nederland (onderdeel van De Rijksoverheid) is één van de grootste werkgevers van ons land. De kans dat jij jouw baan bij de Rijksoverheid vindt is dan ook behoorlijk groot.'
          .untranslated,
  image: Image.asset(path: MockAssets.logo_rijksoverheid),
);

final _kJustisOrganization = Organization(
  //id: kJusticeId,
  legalName: 'Ministerie van Justitie en Veiligheid'.untranslated,
  displayName: 'Justis'.untranslated,
  //type: 'Overheid'.untranslated,
  description:
      'Screeningsautoriteit Justis beoordeelt de betrouwbaarheid van personen en organisaties ter bevordering van een veilige en rechtvaardige samenleving.'
          .untranslated,
  image: Image.asset(path: MockAssets.logo_rijksoverheid),
);

final _kMarketPlaceOrganization = Organization(
  //id: kMarketplaceId,
  legalName: 'Online Marketplace'.untranslated,
  displayName: 'Online Marketplace'.untranslated,
  //type: 'Webwinkel'.untranslated,
  description: 'De winkel van ons allemaal.'.untranslated,
  image: Image.asset(path: MockAssets.logo_ecommerce),
  // department: 'Afdeling online marketing'.untranslated,
  city: 'Utrecht'.untranslated,
  countryCode: 'nl',
  webUrl: 'https://online-marketplace.nl',
);

final _kBarOrganization = Organization(
  //id: kBarId,
  legalName: 'Cafe de Dobbelaar'.untranslated,
  displayName: 'Cafe de Dobbelaar'.untranslated,
  //type: 'Horeca'.untranslated,
  description: 'Familiecafe sinds 1984.'.untranslated,
  image: Image.asset(path: MockAssets.logo_bar),
);

final _kHealthInsurerOrganization = Organization(
  //id: kHealthInsuranceId,
  legalName: 'Zorgverzekeraar Z'.untranslated,
  displayName: 'Zorgverzekeraar Z'.untranslated,
  //type: 'Zorgverlener'.untranslated,
  description:
      'Of het nu gaat om het regelen van zorg, het betalen van zorg of een gezond leven. Zorgverzekeraar Z zet zich elke dag in voor de gezondheid van haar klanten.'
          .untranslated,
  image: Image.asset(path: MockAssets.logo_zorgverzekeraar_z),
);

final _kHousingCorporationOrganization = Organization(
  //id: kHousingCorpId,
  legalName: 'BeterWonen'.untranslated,
  displayName: 'BeterWonen'.untranslated,
  //type: 'Wooncorporatie'.untranslated,
  description: 'Moderne woningen voor iedereen in de Gemeente Den Haag en omstreken.'.untranslated,
  image: Image.asset(path: MockAssets.logo_housing_corp),
  webUrl: 'https://beterwonen.nl',
  countryCode: 'nl',
  city: 'Den Haag'.untranslated,
  // department: 'Secretariaat'.untranslated,
);

final _kCarRentalOrganization = Organization(
  //id: kCarRentalId,
  legalName: 'CarRental'.untranslated,
  displayName: 'CarRental'.untranslated,
  //type: 'Autoverhuur'.untranslated,
  description: 'Betrouwbaar huren.'.untranslated,
  image: Image.asset(path: MockAssets.logo_car_rental),
);

final _kFirstAidOrganization = Organization(
  //id: 'first_aid',
  legalName: 'Healthcare Facility'.untranslated,
  displayName: 'Healthcare Facility'.untranslated,
  //type: 'Zorgverlener'.untranslated,
  description:
      'Deze Healthcare Facility is fictief ter invulling van de Demo. Dit kan een zorginstelling zijn in Nederland of in het buitenland.'
          .untranslated,
  image: Image.asset(path: MockAssets.logo_first_aid),
);

final _kMunicipalityTheHagueOrganization = Organization(
  //id: kMunicipalityTheHagueId,
  legalName: 'Gemeente Den Haag'.untranslated,
  displayName: 'Gemeente Den Haag'.untranslated,
  //type: 'Gemeente'.untranslated,
  description:
      'Den Haag is een unieke stad waar we allemaal trots op zijn. Nieuwsgierig, divers en vol vertrouwen. Vrede en Recht.'
          .untranslated,
  image: Image.asset(path: MockAssets.logo_den_haag),
  // department: 'Parkeren'.untranslated,
  city: 'Den Haag'.untranslated,
  countryCode: 'nl',
  webUrl: 'https://www.denhaag.nl',
);

final _kBankOrganization = Organization(
  //id: kBankId,
  legalName: 'Jouw Bank'.untranslated,
  displayName: 'Jouw Bank'.untranslated,
  //type: 'Bank'.untranslated,
  description: 'Maak het leven makkelijk. Regel je financieën digitaal met Jouw Bank.'.untranslated,
  image: Image.asset(path: MockAssets.logo_bank),
  // department: 'Klantenservice'.untranslated,
  countryCode: 'nl',
  city: 'Amsterdam'.untranslated,
  webUrl: 'https://jouwbank.nl',
);

final _kMonkeyBikeOrganization = Organization(
  //id: kMonkeyBikeId,
  legalName: 'MonkeyBike'.untranslated,
  displayName: 'MonkeyBike'.untranslated,
  //type: 'Bezorgdienst'.untranslated,
  description: 'Razendsnel jouw boodschappen of bestelling bij jouw thuis. Altijd binnen 10 minuten.'.untranslated,
  image: Image.asset(path: MockAssets.logo_monkeybike),
  // department: 'Online marketing'.untranslated,
  countryCode: 'nl',
  city: 'Groningen'.untranslated,
  webUrl: 'https://flitsbezorger-monkeybike.nl',
  kvk: '3945-2932',
);

final _kPharmacyOrganization = Organization(
  //id: kPharmacyId,
  legalName: 'De Noord Apotheek'.untranslated,
  displayName: 'Apotheek'.untranslated,
  //type: 'Apotheek'.untranslated,
  description: 'Al meer dan 25 jaar jouw betrouwbare apotheek.'.untranslated,
  image: Image.asset(path: MockAssets.logo_zorgverzekeraar_z),
  kvk: '1234-1234',
  city: 'Den Haag'.untranslated,
  countryCode: 'nl',
  webUrl: 'https://denoordapotheek.nl',
);
