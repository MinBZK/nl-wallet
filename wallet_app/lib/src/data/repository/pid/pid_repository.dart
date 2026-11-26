import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/issuance/continue_pid_issuance_result.dart';

abstract class PidRepository {
  Future<String> getPidIssuanceUrl();

  Future<String> getPidRenewalUrl();

  /// Continue the pidIssuance, returns a preview of all the attributes that will be added if the pid is accepted.
  /// When the issuer does not provide previews, the pid has to be requested with [issuePidForReview] instead.
  Future<ContinuePidIssuanceResult> continuePidIssuance(String uri);

  Future<TransferState> acceptIssuance(String pin);

  /// Requests the pid from an issuer that does not provide previews, returns its attributes for review.
  /// The pid is not added to the wallet until [commitIssuedPid] is called.
  Future<List<DataAttribute>> issuePidForReview(String pin);

  /// Adds the pid returned by [issuePidForReview] to the wallet.
  Future<TransferState> commitIssuedPid(String pin);
}

/// Indicator on whether the user has the option to transfer a previous wallet to this device
enum TransferState { available, unavailable }
