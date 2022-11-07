import '../../../domain/model/data_attribute.dart';
import 'wallet_card_data_attribute_repository.dart';

class MockWalletCardDataAttributeRepository implements WalletCardDataAttributeRepository {
  MockWalletCardDataAttributeRepository();

  @override
  Future<List<DataAttribute>> getAll(String cardId) async {
    switch (cardId) {
      case '1':
        return _kMockAllDataAttributes;
      case '2':
        return _kMockAllDataAttributes;
      default:
        throw UnimplementedError();
    }
  }
}

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
