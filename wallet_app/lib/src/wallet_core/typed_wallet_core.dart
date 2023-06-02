
import 'package:core_domain/core_domain.dart';

abstract class TypedWalletCore {
  Future<PinValidationResult> isValidPin(String pin);

  Future<void> register(String pin);

  Future<bool> isRegistered();

  Future<String> getDigidAuthUrl();

  Stream<UriFlowEvent> processUri(Uri uri);
}
