import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../environment.dart';
import '../../util/extension/build_context_extension.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/setting/switch_setting_row.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';
import 'bloc/developer_settings_bloc.dart';

class DeveloperSettingsScreen extends StatelessWidget {
  const DeveloperSettingsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    if (!Environment.enableWrprcStrictModeToggle) return const SizedBox.shrink();
    return Scaffold(
      appBar: WalletAppBar(title: TitleText(context.l10n.developerSettingsScreenTitle)),
      body: SafeArea(
        child: BlocBuilder<DeveloperSettingsBloc, DeveloperSettingsState>(
          builder: _buildContent,
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context, DeveloperSettingsState state) {
    return Column(
      children: [
        Expanded(
          child: WalletScrollbar(
            child: ListView(
              children: [
                Padding(
                  padding: kDefaultTitlePadding,
                  child: TitleText(context.l10n.developerSettingsScreenTitle),
                ),
                Padding(
                  padding: const EdgeInsets.only(left: 16, right: 16, bottom: 24),
                  child: BodyText(context.l10n.developerSettingsScreenDisclaimer),
                ),
                SwitchSettingRow(
                  label: Text(context.l10n.developerSettingsScreenWrprcStrictMode),
                  subtitle: Text(context.l10n.developerSettingsScreenWrprcStrictModeDescription),
                  value: state.wrprcStrictMode ?? true,
                  onChanged: state.canToggleWrprcStrictMode
                      ? (enabled) => context.read<DeveloperSettingsBloc>().add(
                          DeveloperSettingsWrprcStrictModeToggled(enabled: enabled),
                        )
                      : null,
                  dividerSide: .both,
                ),
                if (state.hasError)
                  Semantics(
                    liveRegion: true,
                    child: Padding(
                      padding: const EdgeInsets.all(16),
                      child: BodyText(
                        context.l10n.developerSettingsScreenError,
                        style: context.textTheme.bodyLarge?.copyWith(color: context.colorScheme.error),
                      ),
                    ),
                  ),
                const SizedBox(height: 24),
              ],
            ),
          ),
        ),
        const BottomBackButton(),
      ],
    );
  }
}
