import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/help/topic_block.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/common/widget/loading_indicator.dart';
import 'package:wallet/src/feature/error/error_page.dart';
import 'package:wallet/src/feature/help/argument/help_topic_screen_argument.dart';
import 'package:wallet/src/feature/help/bloc/help_topic_bloc.dart';
import 'package:wallet/src/feature/help/help_topic_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';

class MockHelpTopicBloc extends MockBloc<HelpTopicEvent, HelpTopicState> implements HelpTopicBloc {}

const _argument = HelpTopicScreenArgument(topicId: 'what_is_wallet');

/// Mirrors the parsed blocks of the real `what_is_wallet` help topic so the
/// golden keeps rendering representative, full-length content.
const _loadedState = HelpTopicLoadSuccess(
  title: 'What is NL Wallet?',
  blocks: [
    TopicParagraphBlock('NL Wallet is a digital wallet for your cards. You can use it to share data.'),
    TopicHeadingBlock('How it works'),
    TopicBulletListBlock([
      'You can add cards with your data.',
      'You share only what is needed.',
      'Your data is shared only after you agree.',
    ]),
    TopicHeadingBlock('Demo or real wallet'),
    TopicBulletListBlock([
      'In the demo, you see how cards and sharing work.',
      'The demo uses sample data.',
      'The demo is not a real wallet with your data.',
      'In a real wallet, you share real data with organisations.',
    ]),
    TopicHeadingBlock('What can NL Wallet be used for?'),
    TopicBulletListBlock([
      'You can add cards, like a Personal data card and an Address card.',
      'You can share only the details that are needed.',
      'You can stop before anything is shared.',
    ]),
    TopicReferenceBlock([
      TopicReferenceLink(label: 'I do not know what NL Wallet is', topicId: 'dont_know_wallet'),
      TopicReferenceLink(label: 'What can NL Wallet be used for?', topicId: 'what_is_wallet_used_for'),
      TopicReferenceLink(label: 'Is the NL Wallet demo real?', topicId: 'is_demo_real'),
      TopicReferenceLink(label: 'How does sharing work in NL Wallet?', topicId: 'how_sharing_works'),
    ]),
  ],
);

Future<void> _pumpScreen(
  WidgetTester tester,
  HelpTopicState state, {
  Brightness brightness = Brightness.light,
  Size surfaceSize = iphoneXSize,
}) {
  return tester.pumpWidgetWithAppWrapper(
    const HelpTopicScreen(argument: _argument).withState<HelpTopicBloc, HelpTopicState>(MockHelpTopicBloc(), state),
    brightness: brightness,
    surfaceSize: surfaceSize,
  );
}

void main() {
  group('HelpTopicScreen body states', () {
    testWidgets('shows loading indicator while the bloc is loading', (tester) async {
      await _pumpScreen(tester, const HelpTopicLoadInProgress());
      expect(find.byType(LoadingIndicator), findsOneWidget);
    });

    testWidgets('renders topic title and blocks once loaded', (tester) async {
      await _pumpScreen(tester, _loadedState);

      // Title appears both in the app bar and as the body hero — match at least once.
      expect(find.text('What is NL Wallet?'), findsWidgets);
      expect(find.text('How it works'), findsOneWidget);
      expect(find.textContaining('NL Wallet is a digital wallet for your cards.'), findsOneWidget);
    });

    testWidgets('renders an ErrorPage on load failure', (tester) async {
      await _pumpScreen(
        tester,
        const HelpTopicLoadFailure(GenericError('asset missing', sourceError: 'asset missing')),
      );

      expect(find.byType(ErrorPage), findsOneWidget);
      expect(find.byType(LoadingIndicator), findsNothing);
    });
  });

  group('goldens', () {
    testGoldens('HelpTopicScreen loaded light', (tester) async {
      await _pumpScreen(tester, _loadedState);
      await screenMatchesGolden('topic.light');
    });

    testGoldens('HelpTopicScreen loaded dark', (tester) async {
      await _pumpScreen(tester, _loadedState, brightness: Brightness.dark);
      await screenMatchesGolden('topic.dark');
    });

    testGoldens('HelpTopicScreen loaded light - landscape', (tester) async {
      await _pumpScreen(tester, _loadedState, surfaceSize: iphoneXSizeLandscape);
      await screenMatchesGolden('topic.light.landscape');
    });

    testGoldens('HelpTopicScreen loading light', (tester) async {
      await _pumpScreen(tester, const HelpTopicLoadInProgress());
      await screenMatchesGolden('topic.loading.light');
    });
  });
}
