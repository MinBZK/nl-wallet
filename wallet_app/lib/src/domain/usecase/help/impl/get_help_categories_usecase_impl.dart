import 'dart:ui';

import '../../../../data/repository/help/help_content_repository.dart';
import '../../../model/help/help_category.dart';
import '../../../model/result/result.dart';
import '../get_help_categories_usecase.dart';

class GetHelpCategoriesUseCaseImpl extends GetHelpCategoriesUseCase {
  final HelpContentRepository _helpContentRepository;

  GetHelpCategoriesUseCaseImpl(this._helpContentRepository);

  @override
  Future<Result<List<HelpCategory>>> invoke(Locale locale) {
    return tryCatch(
      () => _helpContentRepository.getCategories(locale),
      'Failed to load help categories',
    );
  }
}
