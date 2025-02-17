import '../../../../data/repository/uri/uri_repository.dart';
import '../../../model/navigation/navigation_request.dart';
import '../../../model/result/result.dart';
import '../decode_uri_usecase.dart';

class DecodeUriUseCaseImpl extends DecodeUriUseCase {
  final UriRepository _uriRepository;

  DecodeUriUseCaseImpl(this._uriRepository);

  @override
  Future<Result<NavigationRequest>> invoke(Uri uri) async {
    return tryCatch(
      () async => _uriRepository.processUri(uri),
      'Failed to decode uri: $uri',
    );
  }
}
