import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/animated_list_model.dart';

import '../../../wallet_app_test_widget.dart';

void main() {
  group('AnimatedListModel', () {
    late GlobalKey<AnimatedListState> listKey;
    late List<int> removedItems;
    late AnimatedListModel<int> model;

    Widget removedItemBuilder(int item, BuildContext context, Animation<double> animation) {
      removedItems.add(item);
      return Container();
    }

    setUp(() {
      listKey = GlobalKey<AnimatedListState>();
      removedItems = [];
      model = AnimatedListModel<int>(
        listKey: listKey,
        removedItemBuilder: removedItemBuilder,
        initialItems: [1, 2, 3],
      );
    });

    test('initialization', () {
      expect(model.length, 3);
      expect(model[0], 1);
      expect(model[1], 2);
      expect(model[2], 3);
    });

    test('indexOf', () {
      expect(model.indexOf(2), 1);
      expect(model.indexOf(10), -1);
    });

    testWidgets('insert updates list and calls AnimatedListState', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        AnimatedList(
          key: listKey,
          initialItemCount: model.length,
          itemBuilder: (context, index, animation) => Text(model[index].toString()),
        ),
      );

      model.insert(1, 10);
      expect(model.length, 4);
      expect(model[1], 10);

      await tester.pump();
      expect(find.text('10'), findsOneWidget);
    });

    testWidgets('removeAt updates list and calls AnimatedListState', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        AnimatedList(
          key: listKey,
          initialItemCount: model.length,
          itemBuilder: (context, index, animation) => Text(model[index].toString()),
        ),
      );

      final removed = model.removeAt(1);
      expect(removed, 2);
      expect(model.length, 2);

      await tester.pump();
      // The removed item builder is called by the AnimatedListState when removeItem is called.
      expect(removedItems, contains(2));
    });
  });
}
