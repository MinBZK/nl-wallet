part of 'mock_organization_datasource.dart';

const _kOrganizations = [
  _kRvigOrganization,
  _kRdwOrganization,
  _kDuoOrganization,
  _kEmployerOrganization,
  _kJustisOrganization,
  _kMarketPlaceOrganization,
  _kBarOrganization,
  _kHealthInsurerOrganization,
  _kHousingCorporationOrganization,
  _kCarRentalOrganization,
  _kFirstAidOrganization,
  _kMunicipalityDelftOrganization,
  _kBankOrganization,
  _kMonkeyBikeOrganization,
  _kPharmacyOrganization,
];

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

const _kRvigOrganization = Organization(
  id: kRvigId,
  name: 'Rijksdienst voor Identiteitsgegevens',
  shortName: 'Rijksdienst voor Identiteitsgegevens',
  category: 'Overheid',
  description:
      'Rijksdienst voor Identiteitsgegevens is de autoriteit en regisseur van het veilig en betrouwbaar gebruik van identiteitsgegevens.',
  logoUrl: WalletAssets.logo_rijksoverheid,
);

const _kRdwOrganization = Organization(
  id: kRdwId,
  name: 'Rijksdienst voor het Wegverkeer (RDW)',
  shortName: 'RDW',
  category: 'Overheid',
  logoUrl: WalletAssets.logo_rdw,
  description:
      'De Rijksdienst voor het Wegverkeer (RDW) draagt bij aan een veilig, schoon, economisch en geordend wegverkeer.',
);

const _kDuoOrganization = Organization(
  id: kDuoId,
  name: 'Dienst Uitvoering Onderwijs (DUO)',
  shortName: 'DUO',
  category: 'Overheid',
  logoUrl: WalletAssets.logo_rijksoverheid,
  description:
      'Dienst Uitvoering Onderwijs (DUO) verzorgt onderwijs en ontwikkeling in opdracht van het Nederlandse ministerie van Onderwijs, Cultuur en Wetenschap.',
);

const _kEmployerOrganization = Organization(
  id: kEmployerId,
  name: 'Werken voor Nederland',
  shortName: 'Werken voor Nederland',
  category: 'Bedrijf',
  description:
      'Werken voor Nederland (onderdeel van De Rijksoverheid) is één van de grootste werkgevers van ons land. De kans dat jij jouw baan bij de Rijksoverheid vindt is dan ook behoorlijk groot.',
  logoUrl: WalletAssets.logo_rijksoverheid,
);

const _kJustisOrganization = Organization(
  id: kJusticeId,
  name: 'Ministerie van Justitie en Veiligheid',
  shortName: 'Justis',
  category: 'Overheid',
  description:
      'Screeningsautoriteit Justis beoordeelt de betrouwbaarheid van personen en organisaties ter bevordering van een veilige en rechtvaardige samenleving.',
  logoUrl: WalletAssets.logo_rijksoverheid,
);

const _kMarketPlaceOrganization = Organization(
  id: kMarketplaceId,
  name: 'Online Marketplace',
  shortName: 'Online Marketplace',
  category: 'Webwinkel',
  description: 'De winkel van ons allemaal.',
  logoUrl: WalletAssets.logo_ecommerce,
  department: 'Afdeling online marketing',
  location: 'Utrecht, Nederland',
  webUrl: 'https://online-marketplace.nl',
);

const _kBarOrganization = Organization(
  id: kBarId,
  name: 'Cafe de Dobbelaar',
  shortName: 'Cafe de Dobbelaar',
  category: 'Horeca',
  description: 'Familiecafe sinds 1984.',
  logoUrl: WalletAssets.logo_bar,
);

const _kHealthInsurerOrganization = Organization(
  id: kHealthInsuranceId,
  name: 'Zorgverzekeraar Z',
  shortName: 'Zorgverzekeraar Z',
  category: 'Zorgverlener',
  description:
      'Of het nu gaat om het regelen van zorg, het betalen van zorg of een gezond leven. Zorgverzekeraar Z zet zich elke dag in voor de gezondheid van haar klanten.',
  logoUrl: WalletAssets.logo_zorgverzekeraar_z,
);

const _kHousingCorporationOrganization = Organization(
  id: kHousingCorpId,
  name: 'BeterWonen',
  shortName: 'BetwerWonen',
  category: 'Wooncorporatie',
  description: 'Moderne woningen voor iedereen in de Gemeente Den Haag en omstreken.',
  logoUrl: WalletAssets.logo_housing_corp,
  webUrl: 'https://beterwonen.nl',
  location: 'Den Haag, Nederland',
  department: 'Secretariaat',
);

const _kCarRentalOrganization = Organization(
  id: kCarRentalId,
  name: 'CarRental',
  shortName: 'CarRental',
  category: 'Autoverhuur',
  description: 'Betrouwbaar huren.',
  logoUrl: WalletAssets.logo_car_rental,
);

const _kFirstAidOrganization = Organization(
  id: 'first_aid',
  name: 'Healthcare Facility',
  shortName: 'Healthcare Facility',
  category: 'Zorgverlener',
  description:
      'Deze Healthcare Facility is fictief ter invulling van de Demo. Dit kan een zorginstelling zijn in Nederland of in het buitenland.',
  logoUrl: WalletAssets.logo_first_aid,
);

const _kMunicipalityDelftOrganization = Organization(
  id: kMunicipalityTheHagueId,
  name: 'Gemeente Den Haag',
  shortName: 'Gemeente Den Haag',
  category: 'Gemeente',
  description:
      'Den Haag is een unieke stad waar we allemaal trots op zijn. Nieuwsgierig, divers en vol vertrouwen. Vrede en Recht.',
  logoUrl: WalletAssets.logo_den_haag,
  department: 'Parkeren',
  location: 'Den Haag, Nederland',
  webUrl: 'https://www.denhaag.nl',
);

const _kBankOrganization = Organization(
  id: kBankId,
  name: 'Jouw Bank',
  shortName: 'Jouw Bank',
  category: 'Bank',
  description: 'Maak het leven makkelijk. Regel je financieën digitaal met Jouw Bank.',
  logoUrl: WalletAssets.logo_bank,
  department: 'Klantenservice',
  location: 'Amsterdam, Nederland',
  webUrl: 'https://jouwbank.nl',
);

const _kMonkeyBikeOrganization = Organization(
  id: kMonkeyBikeId,
  name: 'MonkeyBike',
  shortName: 'MonkeyBike',
  category: 'Bezorgdienst',
  description: 'Razendsnel jouw boodschappen of bestelling bij jouw thuis. Altijd binnen 10 minuten.',
  logoUrl: WalletAssets.logo_monkeybike,
  department: 'Online marketing',
  location: 'Groningen, Land',
  webUrl: 'https://flitsbezorger-monkeybike.nl',
  //companyInfo: 'KVK: 3945-2932',
);

const _kPharmacyOrganization = Organization(
  id: kPharmacyId,
  name: 'De Noord Apotheek',
  shortName: 'Apotheek',
  category: 'Apotheek',
  description: 'Al meer dan 25 jaar jouw betrouwbare apotheek.',
  logoUrl: WalletAssets.logo_zorgverzekeraar_z,
  department: 'KVK: 1234-1234',
  location: 'Den Haag, Nederland',
  webUrl: 'https://denoordapotheek.nl',
);
