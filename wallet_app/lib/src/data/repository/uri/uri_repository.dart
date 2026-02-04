import '../../../domain/model/navigation/navigation_request.dart';

abstract class UriRepository {
  Future<NavigationRequest> processUri(Uri inputUri);
}
