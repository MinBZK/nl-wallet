import '../../../domain/model/wallet_card.dart';
import '../../../wallet_constants.dart';
import 'wallet_card_repository.dart';

class MockWalletCardRepository implements WalletCardRepository {
  MockWalletCardRepository();

  @override
  Future<List<WalletCard>> getWalletCards() async {
    await Future.delayed(kDefaultMockDelay);
    return [
      const WalletCard(
        id: '2',
        title: 'Rijbewijs',
        info: 'Categorie AM, B, C1, BE',
        logoImage: 'assets/non-free/images/logo_nl_driving_license.png',
        backgroundImage: 'assets/images/bg_nl_driving_license.png',
      ),
      const WalletCard(
        id: '1',
        title: 'Paspoort',
        info: 'Koninkrijk der Nederlanden',
        logoImage: 'assets/non-free/images/logo_nl_passport.png',
        backgroundImage: 'assets/images/bg_nl_passport.png',
      ),
    ];
  }
}
