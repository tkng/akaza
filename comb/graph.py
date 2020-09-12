import logging
import sys
from logging import Logger
from typing import Dict, List

import jaconv

from comb.language_model import LanguageModel
from comb.node import Node
from comb.system_dict import SystemDict
from comb.user_language_model import UserLanguageModel


class Graph:
    logger: Logger
    d: Dict[int, List[Node]]

    def __init__(self, size: int, logger=logging.getLogger(__name__)):
        self.d = {
            0: [Node(start_pos=-9999, word='<S>', yomi='<S>')],
            size + 1: [
                Node(start_pos=size, word='</S>', yomi='</S>')],
        }
        self.logger = logger

    def __len__(self) -> int:
        return len(self.d)

    def __repr__(self) -> str:
        s = ''
        for i in sorted(self.d.keys()):
            if i in self.d:
                s += f"{i}:\n"
                s += "\n".join(["\t" + str(x) for x in sorted(self.d[i], key=lambda x: x.cost)]) + "\n"
        return s

    def append(self, index: int, node: Node) -> None:
        if index not in self.d:
            self.d[index] = []
        # print(f"graph[{j}]={graph[j]} graph={graph}")
        self.d[index].append(node)

    def get_items(self):
        for i in sorted(self.d.keys()):
            if i == 0:  # skip bos
                continue
            yield self.d[i]

    def get_item(self, i: int) -> List[Node]:
        return self.d[i]

    def dump(self, path: str):
        with open(path, 'w') as fp:
            fp.write("""digraph graph_name {\n""")
            fp.write("""  graph [\n""")
            fp.write("""    charset="utf-8"\n""")
            fp.write("""  ]\n""")
            for i, nodes in self.d.items():
                for node in nodes:
                    fp.write(f"  {node.start_pos} -> {i} [label=\"{node.word}:"
                             f" {node.cost}: node={node.calc_node_cost()} {node.prev.word if node.prev else '-'}\"]\n")
            fp.write("""}\n""")

    def get_eos(self):
        return self.d[max(self.d.keys())][0]

    def get_bos(self):
        return self.d[0][0]


def lookup(s, system_dict: SystemDict, user_language_model: UserLanguageModel):
    for i in range(0, len(s)):
        yomi = s[i:]
        # print(f"YOMI:::: {yomi}")
        words = system_dict.prefixes(yomi)
        if len(words) > 0:
            # print(f"YOMI:::: {yomi} {words}")
            for word in words:
                kanjis = system_dict[word]
                if word not in kanjis:
                    kanjis.append(word)

                kata = jaconv.hira2kata(word)
                if kata not in kanjis:
                    kanjis.append(kata)

                yield word, kanjis

            if yomi not in words and user_language_model.get_unigram_cost(yomi):
                # システム辞書に入ってないがユーザー言語モデルには入っているという場合は候補にいれる。
                kanjis = [yomi]

                kata = jaconv.hira2kata(word)
                if kata not in kanjis:
                    kanjis.append(kata)

                yield yomi, kanjis
        else:
            # print(f"YOMI~~~~:::: {yomi}")
            targets = [yomi[0]]
            hira = jaconv.hira2kata(yomi[0])
            if hira not in targets:
                targets.append(hira)
            yield yomi[0], targets


# n文字目でおわる単語リストを作成する
def graph_construct(s, ht, force_selected_clause: List[slice] = None) -> Graph:
    graph = Graph(size=len(s))

    if force_selected_clause:
        for force_slice in force_selected_clause:
            # 強制的に範囲を指定されている場合。
            # substr は「読み」であろう。
            # word は「漢字」であろう。
            yomi = s[force_slice]
            i = force_slice.start
            j = force_slice.stop
            # print(f"XXXX={s} {force_slice} {yomi}")
            if yomi in ht:
                # print(f"YOMI YOMI: {yomi} {ht[yomi]}")
                for kanji in ht[yomi]:
                    node = Node(i, kanji, yomi)
                    graph.append(index=j, node=node)
            else:
                # print(f"NO YOMI: {yomi}")
                if len(yomi) == 0:
                    raise AssertionError(f"len(yomi) should not be 0. {s}, {force_slice}")
                node = Node(i, yomi, yomi)
                graph.append(index=j, node=node)
    else:
        for i in range(0, len(s)):
            # print(f"LOOP {i}")
            # for i in range(0, len(s)):
            for j in range(i + 1, len(s) + 1):
                # substr は「読み」であろう。
                # word は「漢字」であろう。
                yomi = s[i:j]
                if yomi in ht:
                    # print(f"YOMI YOMI: {yomi} {ht[yomi]}")
                    for kanji in ht[yomi]:
                        node = Node(i, kanji, yomi)
                        graph.append(index=j, node=node)
                else:
                    # print(f"NO YOMI: {yomi}")
                    pass
                    # graph.append(j, Node(j, yomi, yomi, unigram_score=unigram_score, bigram_score=bigram_score))

    return graph


def viterbi(graph: Graph, language_model: LanguageModel) -> List[List[Node]]:
    """
    ビタビアルゴリズムにもとづき、最短の経路を求めて、N-Best 解を求める。
    """
    # BOS にスコアを設定。
    graph.get_bos().cost = 0

    for nodes in graph.get_items():
        # print(f"fFFFF {nodes}")
        for node in nodes:
            # print(f"  PPPP {node}")
            node_cost = language_model.calc_node_cost(node)
            # print(f"  NC {node.word} {node_cost}")
            cost = -sys.maxsize
            shortest_prev = None
            prev_nodes = graph.get_item(node.start_pos)
            if prev_nodes[0].is_bos():
                node.prev = prev_nodes[0]
                node.cost = node_cost
            else:
                for prev_node in prev_nodes:
                    if prev_node.cost is None:
                        logging.error(f"Missing prev_node.cost--- {prev_node}")
                    tmp_cost = prev_node.cost + language_model.calc_bigram_cost(prev_node, node) + node_cost
                    if cost < tmp_cost:
                        cost = tmp_cost
                        shortest_prev = prev_node
                # print(f"    SSSHORTEST: {shortest_prev} in {prev_nodes}")
                node.prev = shortest_prev
                node.cost = cost

    # print(graph)

    # find EOS.
    node = graph.get_eos()
    # node = graph.get_item(len(graph) - 1)[0]

    # print(node)
    result = []
    last_node = None
    while not node.is_bos():
        if node == node.prev:
            print(graph)
            raise AssertionError(f"node==node.prev: {node}")

        if not node.is_eos():
            # 他の候補を追加する。
            nodes = sorted([n for n in graph.get_item(node.start_pos + len(node.yomi)) if node.yomi == n.yomi],
                           key=lambda x: x.cost + language_model.calc_bigram_cost(x, last_node), reverse=True)
            result.append(nodes)

        last_node = node
        node = node.prev
    return list(reversed(result))

