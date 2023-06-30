import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../util/extension/build_context_extension.dart';
import '../../wallet_constants.dart';
import '../common/widget/centered_loading_indicator.dart';
import 'bloc/change_language_bloc.dart';

class ChangeLanguageScreen extends StatelessWidget {
  const ChangeLanguageScreen({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(context.l10n.changeLanguageScreenTitle),
        leading: IconButton(
          key: const Key('changeLanguageScreenBackCta'),
          icon: const Icon(Icons.arrow_back),
          onPressed: () => Navigator.of(context).pop(),
        ),
      ),
      body: BlocBuilder<ChangeLanguageBloc, ChangeLanguageState>(
        builder: (context, state) {
          return switch (state) {
            ChangeLanguageInitial() => const CenteredLoadingIndicator(),
            ChangeLanguageSuccess() => _buildLanguagePicker(context, state),
          };
        },
      ),
    );
  }

  Widget _buildLanguagePicker(BuildContext context, ChangeLanguageSuccess state) {
    return ListView.separated(
      itemBuilder: (c, i) {
        if (i == state.availableLanguages.length) return const SizedBox.shrink(); //Draw final divider
        final language = state.availableLanguages[i];
        final isSelectedLanguage = state.availableLanguages[i].locale == state.selectedLocale;
        return InkWell(
          onTap: () => context.read<ChangeLanguageBloc>().add(ChangeLanguageLocaleSelected(language.locale)),
          child: Container(
            key: ValueKey(language),
            constraints: const BoxConstraints(minHeight: 72),
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            alignment: Alignment.centerLeft,
            child: Row(
              children: [
                Expanded(
                  child: AnimatedDefaultTextStyle(
                    duration: kDefaultAnimationDuration,
                    style: _getRowTextStyle(context, isSelectedLanguage),
                    child: Text(language.name),
                  ),
                ),
                AnimatedOpacity(
                  opacity: isSelectedLanguage ? 1 : 0,
                  duration: kDefaultAnimationDuration,
                  child: Icon(
                    Icons.check,
                    color: context.colorScheme.primary,
                  ),
                ),
              ],
            ),
          ),
        );
      },
      separatorBuilder: (c, i) => const Divider(height: 1),
      itemCount: state.availableLanguages.length + 1, // +1 to Add divider
    );
  }

  TextStyle _getRowTextStyle(BuildContext context, bool isSelected) {
    final baseStyle = context.textTheme.bodyLarge ?? const TextStyle();
    if (!isSelected) return baseStyle;
    return baseStyle.copyWith(color: context.colorScheme.primary, fontWeight: FontWeight.bold);
  }
}
