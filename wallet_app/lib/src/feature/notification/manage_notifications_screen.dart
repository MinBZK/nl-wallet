import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../environment.dart';
import '../../util/cast_util.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/list_extension.dart';
import '../../util/extension/object_extension.dart';
import '../common/widget/button/bottom_back_button.dart';
import '../common/widget/button/icon/back_icon_button.dart';
import '../common/widget/menu_item.dart';
import '../common/widget/setting/switch_setting_row.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import 'bloc/manage_notifications_bloc.dart';
import 'debug/scheduled_notifications_screen.dart';

class ManageNotificationsScreen extends StatelessWidget {
  const ManageNotificationsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.manageNotificationsScreenTitle),
        leading: const BackIconButton(),
      ),
      body: SafeArea(
        child: BlocBuilder<ManageNotificationsBloc, ManageNotificationsState>(
          builder: _buildContent,
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context, ManageNotificationsState state) {
    final bool? pushEnabled = tryCast<ManageNotificationsLoaded>(state)?.pushEnabled;
    return Column(
      children: [
        Expanded(
          child: ListView(
            children: [
              Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: Column(
                      children: [
                        TitleText(context.l10n.manageNotificationsScreenTitle),
                        const SizedBox(height: 8),
                        BodyText(context.l10n.manageNotificationsScreenDescription),
                        const SizedBox(height: 24),
                      ],
                    ),
                  ),
                  SwitchSettingRow(
                    label: Text(context.l10n.manageNotificationsScreenPushSettingTitle),
                    subtitle: Text(context.l10n.manageNotificationsScreenPushSettingSubtitle),
                    value: pushEnabled ?? false,
                    onChanged: pushEnabled != null
                        ? (enabled) {
                            context.read<ManageNotificationsBloc>().add(
                              const ManageNotificationsPushNotificationsToggled(),
                            );
                          }
                        : null,
                    dividerSide: .both,
                  ),
                  MenuItem(
                    label: Text(context.l10n.manageNotificationsScreenDebugTitle),
                    subtitle: Text(context.l10n.manageNotificationsScreenDebugSubtitle),
                    dividerSide: .bottom,
                    onPressed: () => ScheduledNotificationsScreen.show(context),
                  ).takeIf((_) => Environment.showDebugOptions),
                  const SizedBox(height: 16),
                ].nonNullsList,
              ),
            ],
          ),
        ),
        const BottomBackButton(),
      ],
    );
  }
}
