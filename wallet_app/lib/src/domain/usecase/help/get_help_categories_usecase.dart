import 'dart:ui';

import '../../model/help/help_category.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class GetHelpCategoriesUseCase extends WalletUseCase {
  Future<Result<List<HelpCategory>>> invoke(Locale locale);
}
