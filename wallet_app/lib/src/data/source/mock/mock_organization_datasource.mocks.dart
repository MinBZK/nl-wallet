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
const kMunicipalityDelftId = 'municipality_delft';
const kBankId = 'bank';
const kMonkeyBikeId = 'monkey_bike';

const _kRvigOrganization = Organization(
  id: kRvigId,
  name: 'Rijksdienst voor Identiteitsgegevens',
  shortName: 'Rijksdienst voor Identiteitsgegevens',
  category: 'Overheid',
  description:
      'Rijksdienst voor Identiteitsgegevens is de autoriteit en regisseur van het veilig en betrouwbaar gebruik van identiteitsgegevens.',
  logoUrl: 'assets/non-free/images/logo_rijksoverheid.png',
);

const _kRdwOrganization = Organization(
  id: kRdwId,
  name: 'Rijksdienst voor het Wegverkeer (RDW)',
  shortName: 'RDW',
  category: 'Overheid',
  logoUrl: 'assets/non-free/images/logo_rdw.png',
  description:
      'De Rijksdienst voor het Wegverkeer (RDW) draagt bij aan een veilig, schoon, economisch en geordend wegverkeer.',
);

const _kDuoOrganization = Organization(
  id: kDuoId,
  name: 'Dienst Uitvoering Onderwijs (DUO)',
  shortName: 'DUO',
  category: 'Overheid',
  logoUrl: 'assets/non-free/images/logo_rijksoverheid.png',
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
  logoUrl: 'assets/non-free/images/logo_rijksoverheid.png',
);

const _kJustisOrganization = Organization(
  id: kJusticeId,
  name: 'Ministerie van Justitie en Veiligheid',
  shortName: 'Justis',
  category: 'Overheid',
  description:
      'Screeningsautoriteit Justis beoordeelt de betrouwbaarheid van personen en organisaties ter bevordering van een veilige en rechtvaardige samenleving.',
  logoUrl: 'assets/non-free/images/logo_rijksoverheid.png',
);

const _kMarketPlaceOrganization = Organization(
  id: kMarketplaceId,
  name: 'Online Marketplace',
  shortName: 'Online Marketplace',
  category: 'Webwinkel',
  description: 'De winkel van ons allemaal.',
  logoUrl: 'assets/images/logo_ecommerce.png',
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
  logoUrl: 'assets/non-free/images/logo_bar.png',
);

const _kHealthInsurerOrganization = Organization(
  id: kHealthInsuranceId,
  name: 'Zorgverzekeraar Z',
  shortName: 'Zorgverzekeraar Z',
  category: 'Zorgverlener',
  description:
      'Of het nu gaat om het regelen van zorg, het betalen van zorg of een gezond leven. Zorgverzekeraar Z zet zich elke dag in voor de gezondheid van haar klanten.',
  logoUrl: 'assets/images/logo_zorgverzekeraar_z.png',
);

const _kHousingCorporationOrganization = Organization(
  id: kHousingCorpId,
  name: 'BeterWonen',
  shortName: 'BetwerWonen',
  category: 'Wooncorporatie',
  description: 'Moderne woningen voor iedereen in de Gemeente Den Haag en omstreken.',
  logoUrl: 'assets/images/logo_housing_corp.png',
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
  logoUrl: 'assets/non-free/images/logo_car_rental.png',
);

const _kFirstAidOrganization = Organization(
  id: 'first_aid',
  name: 'Healthcare Facility',
  shortName: 'Healthcare Facility',
  category: 'Zorgverlener',
  description:
      'Deze Healthcare Facility is fictief ter invulling van de Demo. Dit kan een zorginstelling zijn in Nederland of in het buitenland.',
  logoUrl: 'assets/non-free/images/logo_first_aid.png',
);

const _kMunicipalityDelftOrganization = Organization(
  id: kMunicipalityDelftId,
  name: 'Gemeente Delft',
  shortName: 'Gemeente Delft',
  category: 'Gemeente',
  description:
      'Delft is ruim 750 jaar oud. De stad dankt haar naam aan het \'delven\' (graven) van de oudste gracht, de Oude Delft. De stad in 5 woorden: monumentale stad van de toekomst. En in 2 woorden: Creating History!',
  logoUrl: 'assets/non-free/images/logo_delft.png',
  department: 'Milieu en gemeente',
  location: 'Delft, Nederland',
  webUrl: 'https://www.delft.nl',
);

const _kBankOrganization = Organization(
  id: kBankId,
  name: 'Jouw Bank',
  shortName: 'Jouw Bank',
  category: 'Bank',
  description: 'Maak het leven makkelijk. Regel je financieën digitaal met Jouw Bank.',
  logoUrl: 'assets/images/logo_bank.png',
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
  logoUrl: 'assets/images/logo_monkeybike.png',
  department: 'Online marketing',
  location: 'Groningen, Land',
  webUrl: 'https://flitsbezorger-monkeybike.nl',
  //companyInfo: 'KVK: 3945-2932',
);
