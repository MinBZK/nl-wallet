import '../../model/navigation/navigation_request.dart';

abstract class DecodeUriUseCase {
  Future<NavigationRequest> invoke(Uri uri);
}
