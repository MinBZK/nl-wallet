import '../../../domain/model/card_front.dart';
import '../../../domain/model/data_attribute.dart';
import '../../../domain/model/issuance_response.dart';
import '../../../domain/model/wallet_card.dart';
import '../../source/organization_datasource.dart';
import '../../source/wallet_datasource.dart';
import 'issuance_response_repository.dart';

class MockIssuanceResponseRepository extends IssuanceResponseRepository {
  final WalletDataSource walletDataSource;
  final OrganizationDataSource organizationDataSource;

  MockIssuanceResponseRepository(this.walletDataSource, this.organizationDataSource);

  @override
  Future<IssuanceResponse> read(String issuanceRequestId) async {
    final organization = (await organizationDataSource.read('rvig'))!;
    switch (issuanceRequestId) {
      case '1':
        if ((await walletDataSource.read(_kMockPassportWalletCard.id)) != null) {
          return IssuanceResponse(organization: organization, cards: []);
        }
        return IssuanceResponse(organization: organization, cards: [_kMockPassportWalletCard]);
      case '2':
        if ((await walletDataSource.read(_kMockLicenseWalletCard.id)) != null) {
          return IssuanceResponse(organization: organization, cards: []);
        }
        return IssuanceResponse(organization: organization, cards: [_kMockLicenseWalletCard]);
    }
    throw UnsupportedError('Unknown issuer: $issuanceRequestId');
  }
}

const _kMockPassportWalletCard = WalletCard(
  id: '1',
  front: _kMockPassportCardFront,
  attributes: _kMockAllDataAttributes,
);

const _kMockLicenseWalletCard = WalletCard(
  id: '2',
  front: _kMockLicenseCardFront,
  attributes: _kMockAllDataAttributes,
);

const _kMockPassportCardFront = CardFront(
  title: 'Paspoort',
  info: 'Koninkrijk der Nederlanden',
  logoImage: 'assets/non-free/images/logo_nl_passport.png',
  backgroundImage: 'assets/images/bg_nl_passport.png',
);

const _kMockLicenseCardFront = CardFront(
  title: 'Rijbewijs',
  info: 'Categorie AM, B, C1, BE',
  logoImage: 'assets/non-free/images/logo_nl_driving_license.png',
  backgroundImage: 'assets/images/bg_nl_driving_license.png',
);

const _kMockAllDataAttributes = [
  DataAttribute(type: 'Image', value: 'assets/non-free/images/person_x.png'),
  DataAttribute(type: 'Naam', value: 'De Bruijn'),
  DataAttribute(type: 'Echtgenote van', value: 'Molenaar'),
  DataAttribute(type: 'Voornamen', value: 'Willeke Liselotte'),
  DataAttribute(type: 'Geboortedatum', value: '10 maart 1965'),
  DataAttribute(type: 'Geboorteplaats', value: 'Delft'),
  DataAttribute(type: 'Geslacht', value: 'Vrouw'),
  DataAttribute(type: 'Lengte', value: '1,75 m'),
  DataAttribute(type: 'Persoonsnummer', value: '9999999999'),
  DataAttribute(type: 'Documentnummer', value: 'SPECI2022'),
  DataAttribute(type: 'Datum verstrekking', value: '20 oktober 2014'),
  DataAttribute(type: 'Geldig tot', value: '20 OKT 2024'),
  DataAttribute(type: 'Type', value: 'P'),
  DataAttribute(type: 'Code', value: 'NL'),
];
