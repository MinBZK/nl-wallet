import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../domain/model/configuration/maintenance_window.dart';
import '../../util/extension/build_context_extension.dart';
import '../../util/extension/date_time_extension.dart';
import '../../util/extension/string_extension.dart';
import '../../util/formatter/datetime/date_formatter.dart';
import '../../wallet_assets.dart';
import '../../wallet_constants.dart';
import '../common/widget/button/button_content.dart';
import '../common/widget/button/icon/help_icon_button.dart';
import '../common/widget/button/primary_button.dart';
import '../common/widget/page_illustration.dart';
import '../common/widget/text/body_text.dart';
import '../common/widget/text/title_text.dart';
import '../common/widget/wallet_app_bar.dart';
import '../common/widget/wallet_scrollbar.dart';

/// Screen displayed during a scheduled maintenance window.
///
/// Shows the user when maintenance started and when it's expected to end.
/// For single-day maintenance, displays the maintenance period within the same day.
/// For multi-day (overnight) maintenance, shows the start and end dates/times separately.
///
/// On Android, provides a "Close App" button to exit the application.
/// On iOS, the normal app exit mechanisms apply.
class MaintenanceScreen extends StatelessWidget {
  /// The maintenance window containing start and end times.
  final MaintenanceWindow maintenanceWindow;

  const MaintenanceScreen({required this.maintenanceWindow, super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: WalletAppBar(
        title: TitleText(context.l10n.maintenanceScreenHeadline),
        actions: const [HelpIconButton()],
      ),
      body: SafeArea(
        child: _buildContent(context),
      ),
    );
  }

  /// Builds the main scrollable content of the maintenance screen.
  ///
  /// Displays the headline, description, and maintenance illustration.
  /// On Android, also includes a close app button at the bottom.
  Widget _buildContent(BuildContext context) {
    return WalletScrollbar(
      child: Column(
        children: [
          Expanded(
            child: CustomScrollView(
              slivers: [
                SliverToBoxAdapter(
                  child: Padding(
                    padding: kDefaultTitlePadding,
                    child: TitleText(context.l10n.maintenanceScreenHeadline),
                  ),
                ),
                SliverToBoxAdapter(
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: BodyText(_getDescription(context)),
                  ),
                ),
                const SliverToBoxAdapter(
                  child: Padding(
                    padding: EdgeInsets.symmetric(vertical: 24),
                    child: PageIllustration(
                      asset: WalletAssets.svg_maintenance,
                    ),
                  ),
                ),
                if (Platform.isAndroid) _buildCloseAppButton(context),
              ],
            ),
          ),
        ],
      ),
    );
  }

  /// Builds a button that allows users to close the app (Android only).
  ///
  /// Displays a "Close" button that calls [SystemNavigator.pop()] to exit the application.
  /// This widget is only shown on Android; iOS users exit via the standard app switcher.
  Widget _buildCloseAppButton(BuildContext context) {
    return SliverFillRemaining(
      hasScrollBody: false,
      child: Column(
        mainAxisAlignment: MainAxisAlignment.end,
        children: [
          const Divider(),
          Padding(
            padding: const EdgeInsets.all(16),
            child: PrimaryButton(
              onPressed: () => SystemNavigator.pop(animated: true),
              icon: const Icon(Icons.close_outlined),
              iconPosition: IconPosition.start,
              mainAxisAlignment: MainAxisAlignment.center,
              text: Text.rich(context.l10n.generalClose.toTextSpan(context)),
            ),
          ),
        ],
      ),
    );
  }

  /// Generates a localized description of the maintenance window.
  ///
  /// Returns different text based on whether the maintenance occurs within a single day
  /// or spans multiple days (overnight). Uses the current locale and date formatting.
  String _getDescription(BuildContext context) {
    final startDateTime = maintenanceWindow.startDateTime;
    final endDateTime = maintenanceWindow.endDateTime;
    final isSameDay = startDateTime.isSameDay(endDateTime);

    final startDateFormatted = DateFormatter.formatMonthDay(context, startDateTime);
    final startTimeFormatted = DateFormatter.formatTime(context, startDateTime);
    final endDateFormatted = DateFormatter.formatMonthDay(context, endDateTime);
    final endTimeFormatted = DateFormatter.formatTime(context, endDateTime);

    if (isSameDay) {
      return context.l10n.maintenanceScreenSameDayDescription(
        startDateFormatted,
        endTimeFormatted,
        startTimeFormatted,
      );
    } else {
      return context.l10n.maintenanceScreenOvernightDescription(
        endDateFormatted,
        endTimeFormatted,
        startDateFormatted,
        startTimeFormatted,
      );
    }
  }
}
