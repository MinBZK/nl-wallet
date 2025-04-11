import 'package:flutter/material.dart';

import '../../../domain/model/app_image_data.dart';
import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/card/metadata/card_display_metadata.dart';
import '../../../domain/model/card/metadata/card_rendering.dart';
import '../../../domain/model/card/wallet_card.dart';
import '../../../domain/model/event/wallet_event.dart';
import '../../../domain/model/flow_progress.dart';
import '../../../domain/model/organization.dart';
import '../../../domain/model/policy/policy.dart';
import '../../../theme/dark_wallet_theme.dart';
import '../../../theme/light_wallet_theme.dart';
import '../../../util/extension/string_extension.dart';
import '../../../wallet_assets.dart';
import '../../card/data/widget/data_privacy_banner.dart';
import '../../common/screen/placeholder_screen.dart';
import '../../common/sheet/confirm_action_sheet.dart';
import '../../common/sheet/error_details_sheet.dart';
import '../../common/sheet/explanation_sheet.dart';
import '../../common/sheet/help_sheet.dart';
import '../../common/widget/activity_summary.dart';
import '../../common/widget/attribute/attribute_row.dart';
import '../../common/widget/bullet_list.dart';
import '../../common/widget/button/animated_visibility_back_button.dart';
import '../../common/widget/button/icon/back_icon_button.dart';
import '../../common/widget/button/icon/help_icon_button.dart';
import '../../common/widget/card/card_logo.dart';
import '../../common/widget/card/shared_attributes_card.dart';
import '../../common/widget/card/wallet_card_item.dart';
import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/fade_in_at_offset.dart';
import '../../common/widget/history/history_section_header.dart';
import '../../common/widget/history/wallet_event_row.dart';
import '../../common/widget/icon_row.dart';
import '../../common/widget/info_row.dart';
import '../../common/widget/loading_indicator.dart';
import '../../common/widget/numbered_list.dart';
import '../../common/widget/pin_field_demo.dart';
import '../../common/widget/pin_header.dart';
import '../../common/widget/policy/policy_row.dart';
import '../../common/widget/policy/policy_section.dart';
import '../../common/widget/select_card_row.dart';
import '../../common/widget/sliver_wallet_app_bar.dart';
import '../../common/widget/stacked_wallet_cards.dart';
import '../../common/widget/status_icon.dart';
import '../../common/widget/stepper_indicator.dart';
import '../../common/widget/version/app_version_text.dart';
import '../../common/widget/wallet_app_bar.dart';
import '../../common/widget/wallet_logo.dart';
import '../../disclosure/widget/card_attribute_row.dart';
import '../../disclosure/widget/disclosure_stop_sheet.dart';
import '../../error/error_screen.dart';
import '../../history/detail/widget/wallet_event_status_header.dart';
import '../../tour/widget/tour_banner.dart';
import '../theme_screen.dart';

const _kMockPurpose = 'Kaart uitgifte';
const _kMockUrl = 'https://www.example.org';
const _kMockOtherKey = 'mock_other';

final _kSampleCardMetaData = [
  CardDisplayMetadata(
    language: Locale('en'),
    name: 'Sample Card',
    rawSummary: 'Subtitle',
    rendering: SimpleCardRendering(
      logoUri: WalletAssets.illustration_digid_failure,
      textColor: DarkWalletTheme.textColor,
    ),
  ),
];

final _kAltSampleCardMetaData = [
  CardDisplayMetadata(
    language: Locale('en'),
    name: 'Alt Sample Card',
    rawSummary: 'Alt Subtitle',
    rendering: SimpleCardRendering(
      logoUri: WalletAssets.logo_card_rijksoverheid,
      textColor: LightWalletTheme.textColor,
    ),
  ),
];

final _kSampleAttributes = [
  DataAttribute(
    key: 'key1',
    label: 'Sample #1'.untranslated,
    value: const StringValue('1'),
    sourceCardDocType: 'sourceCardDocType',
  ),
  DataAttribute(
    key: 'key2',
    label: 'Sample #2'.untranslated,
    value: const StringValue('2'),
    sourceCardDocType: 'sourceCardDocType',
  ),
];

final _kSampleCard = WalletCard(
  id: 'id',
  docType: 'docType',
  metadata: _kSampleCardMetaData,
  attributes: _kSampleAttributes,
  issuer: _kSampleOrganization,
);

final _kAltSampleCard = WalletCard(
  id: 'alt_id',
  docType: 'alt_docType',
  metadata: _kAltSampleCardMetaData,
  attributes: _kSampleAttributes,
  issuer: _kSampleOrganization,
);

final _kSampleOrganization = Organization(
  id: 'id',
  legalName: 'Organization Legal Name'.untranslated,
  displayName: 'Organization Display Name'.untranslated,
  category: 'Category'.untranslated,
  description: 'Organization description'.untranslated,
  logo: const AppAssetImage(WalletAssets.logo_rijksoverheid),
);

final _kSampleIssuanceEvent = WalletEvent.issuance(
  dateTime: DateTime.now(),
  status: EventStatus.success,
  card: _kSampleCard,
);

final _kSampleInteractionAttribute = WalletEvent.disclosure(
  dateTime: DateTime.now(),
  relyingParty: _kSampleOrganization,
  cards: [_kSampleCard],
  status: EventStatus.success,
  policy: const Policy(
    storageDuration: Duration(days: 90),
    dataPurpose: _kMockPurpose,
    dataIsShared: false,
    deletionCanBeRequested: true,
    privacyPolicyUrl: _kMockUrl,
  ),
  purpose: _kMockPurpose.untranslated,
  type: DisclosureType.regular,
);

class OtherStylesTab extends StatelessWidget {
  const OtherStylesTab({super.key});

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 32),
      children: [
        _buildAppBarSection(context),
        _buildSheetSection(context),
        _buildErrorScreensSection(context),
        _buildAttributeSection(context),
        _buildCardSection(context),
        _buildPlaceholderSection(context),
        _buildHistorySection(context),
        _buildPolicySection(context),
        _buildMiscellaneousSection(context),
      ],
    );
  }

  Widget _buildAppBarSection(BuildContext context) {
    return Column(
      children: [
        const ThemeSectionHeader(title: 'App Bars'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'Sliver Wallet App Bar'),
        TextButton(
          onPressed: () => _showSliverWalletAppBarPage(context),
          child: const Text('SliverWalletAppBar'),
        ),
        const ThemeSectionSubHeader(title: 'Wallet App Bar'),
        TextButton(
          onPressed: () => _showWalletAppBarPage(context),
          child: const Text('WalletAppBar'),
        ),
        const ThemeSectionSubHeader(title: 'Wallet App Bar + FadeInAtOffset'),
        TextButton(
          onPressed: () => _showWalletAppBarPageWithFadeInTitle(context),
          child: const Text('WalletAppBar + FadeInAtOffset'),
        ),
      ],
    );
  }

  Widget _buildSheetSection(BuildContext context) {
    return Column(
      children: [
        const ThemeSectionHeader(title: 'Sheets'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'Explanation Sheet'),
        TextButton(
          onPressed: () {
            ExplanationSheet.show(
              context,
              title: 'Title goes here',
              description: 'Description goes here. This is a demo of the ExplanationSheet!',
              closeButtonText: 'Close',
            );
          },
          child: const Text('Explanation Sheet'),
        ),
        const ThemeSectionSubHeader(title: 'Confirm Action Sheet'),
        TextButton(
          onPressed: () {
            ConfirmActionSheet.show(
              context,
              title: 'Title goes here',
              description: 'Description goes here. This is a demo of the ConfirmActionSheet!',
              cancelButtonText: 'cancel',
              confirmButtonText: 'confirm',
            );
          },
          child: const Text('Confirm Action Sheet'),
        ),
        const ThemeSectionSubHeader(title: 'Help Sheet'),
        TextButton(
          onPressed: () {
            HelpSheet.show(
              context,
              errorCode: 'xxyyzz',
              supportCode: '1337',
            );
          },
          child: const Text('Help Sheet'),
        ),
        const ThemeSectionSubHeader(title: 'Error Details Sheet'),
        TextButton(
          onPressed: () => ErrorDetailsSheet.show(context),
          child: const Text('Error Details'),
        ),
        const ThemeSectionSubHeader(title: 'Disclosure Stop Sheet'),
        TextButton(
          onPressed: () {
            DisclosureStopSheet.show(
              context,
              organizationName: 'Organization name'.untranslated,
            );
          },
          child: const Text('Disclosure Stop Sheet'),
        ),
      ],
    );
  }

  Widget _buildErrorScreensSection(BuildContext context) {
    return Column(
      children: [
        const ThemeSectionHeader(title: 'Errors'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'Generic Error Screen'),
        TextButton(
          onPressed: () => ErrorScreen.showGeneric(context),
          child: const Text('Generic Error Screen'),
        ),
        const ThemeSectionSubHeader(title: 'Network Error Screen'),
        TextButton(
          onPressed: () => ErrorScreen.showNetwork(context),
          child: const Text('Network Error Screen'),
        ),
        const ThemeSectionSubHeader(title: 'No Internet Error Screen'),
        TextButton(
          onPressed: () => ErrorScreen.showNoInternet(context),
          child: const Text('No Internet Error Screen'),
        ),
        const ThemeSectionSubHeader(title: 'Device Incompatible Screen'),
        TextButton(
          onPressed: () => ErrorScreen.showDeviceIncompatible(context),
          child: const Text('Device Incompatible Screen'),
        ),
        const ThemeSectionSubHeader(title: 'Session Expired Screen'),
        TextButton(
          onPressed: () => ErrorScreen.showSessionExpired(context),
          child: const Text('Session Expired Screen'),
        ),
      ],
    );
  }

  Widget _buildPlaceholderSection(BuildContext context) {
    return Column(
      children: [
        const ThemeSectionHeader(title: 'Placeholders'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'Generic Placeholder'),
        TextButton(
          onPressed: () => PlaceholderScreen.showGeneric(context, secured: false),
          child: const Text('Generic'),
        ),
        const ThemeSectionSubHeader(title: 'Contract Placeholder'),
        TextButton(
          onPressed: () => PlaceholderScreen.showContract(context, secured: false),
          child: const Text('Contract'),
        ),
      ],
    );
  }

  Widget _buildAttributeSection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const ThemeSectionHeader(title: 'Attributes'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'DataAttributeRow - Type Text'),
        AttributeRow(
          attribute: DataAttribute.untranslated(
            value: const StringValue('This is a DataAttributeRow with type text'),
            label: 'Label',
            sourceCardDocType: 'id',
            key: _kMockOtherKey,
          ),
        ),
        const ThemeSectionSubHeader(title: 'RequestedAttributeRow'),
        AttributeRow(
          attribute: MissingAttribute.untranslated(
            label: 'This is a RequestedAttributeRow',
            key: _kMockOtherKey,
          ),
        ),
        const ThemeSectionSubHeader(title: 'UiAttributeRow'),
        AttributeRow(
          attribute: UiAttribute.untranslated(
            value: const StringValue('This is a UiAttributeRow'),
            key: _kMockOtherKey,
            label: 'Label',
            icon: Icons.remove_red_eye,
          ),
        ),
        const ThemeSectionSubHeader(title: 'CardAttributeRow'),
        CardAttributeRow(
          entry: MapEntry(
            _kSampleCard,
            [
              DataAttribute.untranslated(
                label: 'Voornaam',
                value: const StringValue(''),
                sourceCardDocType: '',
                key: '',
              ),
              DataAttribute.untranslated(
                label: 'Achternaam',
                value: const StringValue(''),
                sourceCardDocType: '',
                key: '',
              ),
              DataAttribute.untranslated(
                label: 'Postcode',
                value: const StringValue(''),
                sourceCardDocType: '',
                key: '',
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildCardSection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const ThemeSectionHeader(title: 'Cards'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'WalletCardItem'),
        const WalletCardItem(
          title: 'Card Title',
          background: DecoratedBox(decoration: BoxDecoration(color: Colors.orangeAccent)),
          subtitle: 'Card subtitle1',
          logo: CardLogo(logo: WalletAssets.logo_card_rijksoverheid),
        ),
        const ThemeSectionSubHeader(title: 'StackedWalletCards'),
        StackedWalletCards(
          cards: [
            _kAltSampleCard,
            _kSampleCard,
          ],
        ),
        const ThemeSectionSubHeader(title: 'SharedWalletCard'),
        SharedAttributesCard(
          card: _kSampleCard,
          attributes: _kSampleCard.attributes,
        ),
      ],
    );
  }

  Widget _buildHistorySection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const ThemeSectionHeader(title: 'History'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'WalletEventRow'),
        WalletEventRow(
          event: WalletEvent.disclosure(
            dateTime: DateTime.now(),
            relyingParty: _kSampleOrganization,
            status: EventStatus.success,
            policy: const Policy(
              storageDuration: Duration(days: 90),
              dataPurpose: _kMockPurpose,
              dataIsShared: false,
              deletionCanBeRequested: true,
              privacyPolicyUrl: _kMockUrl,
            ),
            purpose: _kMockPurpose.untranslated,
            cards: [
              WalletCard(
                id: 'id',
                docType: 'docType',
                metadata: _kSampleCardMetaData,
                attributes: const [],
                issuer: _kSampleOrganization,
              ),
            ],
            type: DisclosureType.regular,
          ),
          onPressed: () {},
        ),
        const ThemeSectionSubHeader(title: 'WalletEventStatusHeader'),
        WalletEventStatusHeader(
          event: WalletEvent.disclosure(
            dateTime: DateTime.now(),
            relyingParty: _kSampleOrganization,
            status: EventStatus.cancelled,
            policy: const Policy(
              storageDuration: Duration(days: 90),
              dataPurpose: _kMockPurpose,
              dataIsShared: false,
              deletionCanBeRequested: true,
              privacyPolicyUrl: _kMockUrl,
            ),
            purpose: _kMockPurpose.untranslated,
            cards: [
              WalletCard(
                id: 'id',
                docType: 'docType',
                metadata: _kSampleCardMetaData,
                attributes: const [],
                issuer: _kSampleOrganization,
              ),
            ],
            type: DisclosureType.regular,
          ),
        ),
        const ThemeSectionSubHeader(title: 'HistorySectionHeader'),
        HistorySectionHeader(dateTime: DateTime.now()),
      ],
    );
  }

  Widget _buildPolicySection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const ThemeSectionHeader(title: 'Policy'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'PolicyRow'),
        const PolicyRow(icon: Icons.alarm, title: 'This is a Policy Row'),
        const ThemeSectionSubHeader(title: 'PolicySection'),
        PolicySection(
          relyingParty: _kSampleOrganization,
          policy: const Policy(
            storageDuration: Duration(days: 90),
            dataPurpose: _kMockPurpose,
            dataIsShared: false,
            deletionCanBeRequested: true,
            privacyPolicyUrl: _kMockUrl,
          ),
        ),
      ],
    );
  }

  Widget _buildMiscellaneousSection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const ThemeSectionHeader(title: 'Miscellaneous'),
        const SizedBox(height: 12),
        const ThemeSectionSubHeader(title: 'DataPrivacyBanner'),
        const DataPrivacyBanner(),
        const ThemeSectionSubHeader(title: 'StepperIndicator'),
        const StepperIndicator(padding: EdgeInsets.zero),
        const ThemeSectionSubHeader(title: 'AnimatedVisibilityBackButton'),
        const AnimatedVisibilityBackButton(visible: true),
        const ThemeSectionSubHeader(title: 'CenteredLoadingIndicator'),
        const CenteredLoadingIndicator(),
        const ThemeSectionSubHeader(title: 'LoadingIndicator'),
        const LoadingIndicator(),
        const ThemeSectionSubHeader(title: 'PinHeader'),
        const PinHeader(title: 'Title', description: 'Description', hasError: false),
        const ThemeSectionSubHeader(title: 'SelectCardRow'),
        SelectCardRow(
          onCardSelectionToggled: (_) {},
          card: WalletCard(
            id: 'row_id',
            docType: 'docType',
            metadata: _kSampleCardMetaData,
            attributes: const [],
            issuer: _kSampleOrganization,
          ),
          isSelected: true,
        ),
        const ThemeSectionSubHeader(title: 'StatusIcon'),
        const StatusIcon(icon: Icons.ac_unit),
        const ThemeSectionSubHeader(title: 'VersionText'),
        const AppVersionText(),
        const ThemeSectionSubHeader(title: 'WalletLogo'),
        const WalletLogo(size: 64),
        const ThemeSectionSubHeader(title: 'IconRow'),
        const IconRow(
          icon: Icon(Icons.ac_unit),
          text: Text('IconRow'),
        ),
        const ThemeSectionSubHeader(title: 'InfoRow'),
        const InfoRow(
          icon: Icons.ac_unit,
          title: Text('Title'),
          subtitle: Text('Subtitle'),
        ),
        const ThemeSectionSubHeader(title: 'PinField'),
        const PinFieldDemo(),
        const ThemeSectionSubHeader(title: 'BulletList'),
        const BulletList(
          items: ['Item 1', 'Item 2', 'Item 3'],
          icon: Icons.ac_unit_outlined,
        ),
        const ThemeSectionSubHeader(title: 'NumberedList'),
        const NumberedList(
          items: ['Item', 'Item', 'Item'],
        ),
        const ThemeSectionSubHeader(title: 'ActivitySummary'),
        ActivitySummary(
          events: [
            _kSampleIssuanceEvent,
            _kSampleIssuanceEvent,
            _kSampleInteractionAttribute,
            _kSampleInteractionAttribute,
            _kSampleInteractionAttribute,
          ],
        ),
        const ThemeSectionSubHeader(title: 'TourBanner'),
        TourBanner(),
      ],
    );
  }

  void _showSliverWalletAppBarPage(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) {
          return Scaffold(
            body: CustomScrollView(
              slivers: [
                SliverWalletAppBar(
                  title: 'Sliver App Bar',
                  scrollController: PrimaryScrollController.maybeOf(context),
                  progress: const FlowProgress(currentStep: 2, totalSteps: 3),
                  leading: const BackIconButton(),
                  actions: const [HelpIconButton()],
                ),
                const SliverFillRemaining(
                  child: Padding(
                    padding: EdgeInsets.symmetric(horizontal: 16),
                    child: Text('Scroll this page to see the collapsing effect'),
                  ),
                ),
              ],
            ),
          );
        },
      ),
    );
  }

  void _showWalletAppBarPage(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) {
          return Scaffold(
            appBar: const WalletAppBar(
              title: Text('WalletAppBar'),
              progress: FlowProgress(currentStep: 2, totalSteps: 8),
              leading: BackIconButton(),
              actions: [HelpIconButton()],
            ),
            body: ListView.builder(
              itemBuilder: (context, index) {
                if (index == 2) {
                  return Container(
                    padding: const EdgeInsets.all(12),
                    alignment: Alignment.center,
                    child: const Text(
                      'This is a more static variant of the custom AppBar without collapse effect',
                      textAlign: TextAlign.center,
                    ),
                  );
                }
                return Container(
                  height: 100,
                  color: index.isOdd ? Colors.greenAccent : Colors.transparent,
                );
              },
              itemCount: 50,
            ),
          );
        },
      ),
    );
  }

  void _showWalletAppBarPageWithFadeInTitle(BuildContext context) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) {
          return Scaffold(
            appBar: const WalletAppBar(
              title: FadeInAtOffset(
                appearOffset: 50,
                visibleOffset: 150,
                child: Text('FadeInAtOffset'),
              ),
              progress: FlowProgress(currentStep: 2, totalSteps: 8),
              leading: BackIconButton(),
              actions: [HelpIconButton()],
            ),
            body: ListView.builder(
              itemBuilder: (context, index) {
                if (index == 2) {
                  return Container(
                    height: 100,
                    padding: const EdgeInsets.all(12),
                    alignment: Alignment.center,
                    child: const Text(
                      'This is the Static WalletAppBar combined with a '
                      'FadeInAtOffset to recreate the collapse and show title effect.',
                      textAlign: TextAlign.center,
                    ),
                  );
                }
                return Container(
                  height: 100,
                  color: index.isOdd ? Colors.greenAccent : Colors.transparent,
                );
              },
              itemCount: 50,
            ),
          );
        },
      ),
    );
  }
}
