import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/issuance/start_issuance_result.dart';

export '../../../domain/model/disclosure/start_disclosure_result.dart';

abstract class IssuanceRepository {
  Future<StartIssuanceResult> startIssuance(String disclosureUri, {required bool isQrCode});

  Future<List<WalletCard>> discloseForIssuance(String pin);

  Future<void> acceptIssuance(String pin, Iterable<WalletCard> cards);

  Future<String?> cancelIssuance();
}
