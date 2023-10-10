import '../../model/navigation/navigation_request.dart';

abstract class DecodeDeeplinkUseCase {
  NavigationRequest? invoke(Uri uri);
}
