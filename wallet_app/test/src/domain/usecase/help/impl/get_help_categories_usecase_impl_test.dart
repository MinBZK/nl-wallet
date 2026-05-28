import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/help/help_category.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/usecase/help/impl/get_help_categories_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockHelpContentRepository mockHelpContentRepository;
  late GetHelpCategoriesUseCaseImpl useCase;

  const locale = Locale('en');

  setUp(() {
    mockHelpContentRepository = MockHelpContentRepository();
    useCase = GetHelpCategoriesUseCaseImpl(mockHelpContentRepository);
  });

  test('invoke returns Success with the categories from the repository', () async {
    const categories = [
      HelpCategory(
        id: 'cat1',
        icon: 'play_arrow',
        title: 'Category 1',
        subtitle: 'Subtitle',
        subcategories: [],
      ),
    ];
    when(mockHelpContentRepository.getCategories(locale)).thenAnswer((_) async => categories);

    final result = await useCase.invoke(locale);

    expect(result, isA<Success<List<HelpCategory>>>());
    expect(result.value, categories);
  });

  test('invoke returns an Error when the repository throws', () async {
    when(mockHelpContentRepository.getCategories(locale)).thenThrow(Exception('yaml missing'));

    final result = await useCase.invoke(locale);

    expect(result, isA<Error<List<HelpCategory>>>());
    expect(result.error, isA<GenericError>());
  });
}
