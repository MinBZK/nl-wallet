import '../../model/result/result.dart';
import '../../model/tour/tour_video.dart';
import '../wallet_usecase.dart';

abstract class FetchTourVideosUseCase extends WalletUseCase {
  Future<Result<List<TourVideo>>> invoke();
}
