import '../../domain/usecase/pin/check_pin_usecase.dart';

extension CheckPinResultExtension on CheckPinResult {
  void when({
    Function(CheckPinResultOk)? onCheckPinResultOk,
    Function(CheckPinResultIncorrect)? onCheckPinResultIncorrectPin,
    Function(CheckPinResultTimeout)? onCheckPinResultTimeout,
    Function(CheckPinResultBlocked)? onCheckPinResultBlocked,
    Function(CheckPinResultGenericError)? onCheckPinResultServerError,
  }) {
    if (this is CheckPinResultOk) {
      onCheckPinResultOk?.call(this as CheckPinResultOk);
    } else if (this is CheckPinResultIncorrect) {
      onCheckPinResultIncorrectPin?.call(this as CheckPinResultIncorrect);
    } else if (this is CheckPinResultTimeout) {
      onCheckPinResultTimeout?.call(this as CheckPinResultTimeout);
    } else if (this is CheckPinResultGenericError) {
      onCheckPinResultServerError?.call(this as CheckPinResultGenericError);
    } else if (this is CheckPinResultBlocked) {
      onCheckPinResultBlocked?.call(this as CheckPinResultBlocked);
    }
  }
}
