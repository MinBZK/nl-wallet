import 'package:collection/collection.dart';

import '../../../feature/verification/model/organization.dart';
import '../organization_datasource.dart';

class MockOrganizationDataSource implements OrganizationDataSource {
  @override
  Future<Organization?> read(String organizationId) async {
    return _kOrganizations.firstWhereOrNull((element) => element.id == organizationId);
  }
}

const _kOrganizations = [
  _kRijksOrganization,
  _kRdwOrganization,
  _kDuoOrganization,
  _kLotteryOrganization,
  _kEmployerOrganization,
  _kJustisOrganization,
  _kMarketPlaceOrganization,
  _kBarOrganization,
  _kHealthInsurerOrganization,
];

const _kRijksOrganization = Organization(
  id: 'rvig',
  name: 'Rijksdienst voor Identiteitsgegevens',
  shortName: 'RvIG',
  description: 'RvIG is de autoriteit en regisseur van het veilig en betrouwbaar gebruik van identiteitsgegevens.',
  logoUrl: 'assets/non-free/images/logo_rijksoverheid.png',
);

const _kRdwOrganization = Organization(
  id: 'rdw',
  name: 'Rijksdienst voor het Wegverkeer (RDW)',
  shortName: 'RDW',
  logoUrl: 'assets/non-free/images/logo_rdw.png',
  description:
      'De Rijksdienst voor het Wegverkeer (RDW) draagt bij aan een veilig, schoon, economisch en geordend wegverkeer.',
);

const _kDuoOrganization = Organization(
  id: 'duo',
  name: 'Dienst Uitvoering Onderwijs (DUO)',
  shortName: 'DUO',
  logoUrl: 'assets/non-free/images/logo_rijksoverheid.png',
  description:
      'Dienst Uitvoering Onderwijs (DUO) verzorgt onderwijs en ontwikkeling in opdracht van het Nederlandse ministerie van Onderwijs, Cultuur en Wetenschap.',
);

const _kLotteryOrganization = Organization(
  id: 'staatsloterij',
  name: 'Nederlandse Staatsloterij',
  shortName: 'Staatsloterij',
  description:
      'Staatsloterij B.V. is een van de dochtervennootschappen van Nederlandse Loterij B.V.[1] De rechtsvoorganger Stichting Exploitatie Nederlandse Staatsloterij (SENS)[2] is in 1992 opgericht en heeft tot 2018 de Staatsloterij georganiseerd.',
  logoUrl: 'assets/non-free/images/logo_staatsloterij.png',
);

const _kEmployerOrganization = Organization(
  id: 'employer_1',
  name: 'Werkgever X',
  shortName: 'Werkgever X',
  description: 'Werkgever X draagt bij aan een betere digitale overheid.',
  logoUrl: 'assets/images/work_logo.png',
);

const _kJustisOrganization = Organization(
  id: 'justis',
  name: 'Ministerie van Justitie en Veiligheid',
  shortName: 'Justis',
  description:
      'Screeningsautoriteit Justis beoordeelt de betrouwbaarheid van personen en organisaties ter bevordering van een veilige en rechtvaardige samenleving.',
  logoUrl: 'assets/non-free/images/logo_rijksoverheid.png',
);

const _kMarketPlaceOrganization = Organization(
  id: 'marketplace',
  name: 'Online Marketplace Y',
  shortName: 'Marketplace Y',
  description: 'Online Marketplace Y is een Nederlands online marktplaats met een hoofdkantoor in Amsterdam.',
  logoUrl: 'assets/non-free/images/logo_ecommerce.png',
);

const _kBarOrganization = Organization(
  id: 'bar',
  name: 'Cafe de Dobbelaar',
  shortName: 'Cafe de Dobbelaar',
  description: 'Poolcafe in Delft.',
  logoUrl: 'assets/non-free/images/logo_bar.png',
);

const _kHealthInsurerOrganization = Organization(
  id: 'health_insurer_1',
  name: 'Zorgverzekeraar Z',
  shortName: 'Zorgverzekeraar Z',
  description:
      'Of het nu gaat om het regelen van zorg, het betalen van zorg of een gezond leven. Zorgverzekeraar Z zet zich elke dag in voor de gezondheid van haar klanten.',
  logoUrl: 'assets/images/logo_zorgverzekeraar_z.png',
);
