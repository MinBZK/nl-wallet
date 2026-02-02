import 'package:flutter/widgets.dart';

import '../../wallet_constants.dart';

typedef RemovedItemBuilder<T> = Widget Function(T item, BuildContext context, Animation<double> animation);

/// Keeps a Dart [List] in sync with an [AnimatedList].
///
/// The [insert] and [removeAt] methods apply to both the internal list and
/// the animated list that belongs to [listKey].
///
/// This class only exposes as much of the Dart List API as is needed by the
/// sample app. More list methods are easily added, however methods that
/// mutate the list must make the same changes to the animated list in terms
/// of [AnimatedListState.insertItem] and [AnimatedListState.removeItem].
///
/// Taken from: https://api.flutter.dev/flutter/widgets/AnimatedList-class.html#widgets.AnimatedList.1
class AnimatedListModel<E> {
  AnimatedListModel({required this.listKey, required this.removedItemBuilder, Iterable<E>? initialItems})
    : items = List<E>.from(initialItems ?? <E>[]);

  final GlobalKey<AnimatedListState> listKey;
  final RemovedItemBuilder<E> removedItemBuilder;
  final List<E> items;

  AnimatedListState? get _animatedList => listKey.currentState;

  void insert(int index, E item) {
    items.insert(index, item);
    _animatedList!.insertItem(index, duration: kDefaultAnimationDuration);
  }

  E removeAt(int index) {
    final E removedItem = items.removeAt(index);
    if (removedItem != null) {
      _animatedList!.removeItem(
        index,
        (BuildContext context, Animation<double> animation) => removedItemBuilder(removedItem, context, animation),
        duration: kDefaultAnimationDuration,
      );
    }
    return removedItem;
  }

  int get length => items.length;

  E operator [](int index) => items[index];

  int indexOf(E item) => items.indexOf(item);
}
