import '../../../domain/model/issue_response.dart';

abstract class IssueResponseRepository {
  Future<IssueResponse> read(String issueRequestId);
}
