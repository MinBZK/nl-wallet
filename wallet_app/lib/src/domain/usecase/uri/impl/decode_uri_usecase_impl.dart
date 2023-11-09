import '../../../../data/repository/uri/uri_repository.dart';
import '../../../model/navigation/navigation_request.dart';
import '../decode_uri_usecase.dart';

class DecodeUriUseCaseImpl implements DecodeUriUseCase {
  final UriRepository _uriRepository;

  DecodeUriUseCaseImpl(this._uriRepository);

  @override
  Future<NavigationRequest> invoke(Uri uri) => _uriRepository.processUri(uri);
}
