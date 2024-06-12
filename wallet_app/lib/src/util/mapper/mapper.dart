abstract class Mapper<I, O> {
  O map(I input);

  List<O> mapList(Iterable<I> input) => input.map(map).toList();
}
