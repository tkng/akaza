from logging import Logger
from typing import List, Any, Dict

import os

import jaconv

from comb import combromkan

from comb.system_dict import SystemDict
from comb.system_language_model import SystemLanguageModel
from comb.user_language_model import UserLanguageModel
from comb.graph import graph_construct, viterbi, lookup
from comb.node import Node
from comb.language_model import LanguageModel
from comb.config import MODEL_DIR
import logging
import marisa_trie
import time

from datetime import date

import re

# 子音だが、N は NN だと「ん」になるので処理しない。
TRAILING_CONSONANT_PATTERN = re.compile(r'^(.*?)([qwrtypsdfghjklzxcvbm]+)$')


class Comb:
    logger: Logger
    dictionaries: List[Any]

    def __init__(self, user_language_model: UserLanguageModel, system_dict: SystemDict,
                 logger: Logger = logging.getLogger(__name__)):
        self.logger = logger
        self.dictionaries = []
        self.user_language_model = user_language_model
        self.system_dict = system_dict

        system_language_model = SystemLanguageModel.create()

        self.language_model = LanguageModel(system_language_model, user_language_model)

    # 連文節変換するバージョン。
    def convert2(self, src: str, force_selected_clause: List[slice] = None) -> List[List[Node]]:
        self.logger.info(f"convert2: {force_selected_clause}")

        if len(src) > 0 and src[0].isupper() and not force_selected_clause:
            # 最初の文字が大文字で、文節の強制指定がない場合、アルファベット強制入力とする。
            return [[
                Node(
                    start_pos=0,
                    word=src,
                    yomi=src,
                )
            ]]

        hiragana: str = combromkan.to_hiragana(src)

        # 末尾の子音を変換対象外とする。
        m = TRAILING_CONSONANT_PATTERN.match(hiragana)
        if m:
            hiragana = m[1]
            consonant = m[2]
            print(f"{hiragana} {consonant}")

        katakana: str = jaconv.hira2kata(hiragana)
        self.logger.info(f"convert: src={src} hiragana={hiragana} katakana={katakana}")

        t0 = time.time()
        ht = dict(lookup(hiragana, self.system_dict))
        graph = graph_construct(hiragana, ht, force_selected_clause)
        self.logger.info(
            f"graph_constructed: src={src} hiragana={hiragana} katakana={katakana}: {time.time() - t0} seconds")
        clauses = viterbi(graph, self.language_model)
        self.logger.info(
            f"converted: src={src} hiragana={hiragana} katakana={katakana}: {time.time() - t0} seconds")

        if m:
            clauses.append([Node(
                start_pos=len(src),
                word=consonant,
                yomi=consonant,
            )])
            return clauses
        else:
            return clauses

    # 連文節しないバージョン(しばらくのあいだ、残しておく。)
    # TODO: remove this.
    def convert(self, src):
        hiragana: str = combromkan.to_hiragana(src)
        katakana: str = jaconv.hira2kata(hiragana)

        self.logger.info(f"convert: src={src} hiragana={hiragana} katakana={katakana}")

        candidates = [[hiragana, hiragana]]

#        for e in self.user_dict.get_candidates(src, hiragana):
#            if e not in candidates:
#                candidates.append(e)

        if hiragana == 'きょう':
            # こういう類の特別なワードは、そのまま記憶してはいけない。。。
            today = date.today()
            for dt in [today.strftime(fmt) for fmt in ['%Y-%m-%d', '%Y年%m月%d日']]:
                candidates.append([dt, dt])

        try:
            ht = dict(lookup(hiragana, self.system_dict))
            graph = graph_construct(hiragana, ht)
            got = viterbi(graph)

            phrase = ''.join([x.word for x in got if not x.is_eos()])

            self.logger.info(f"Got phrase: {phrase}")

            if [phrase, phrase] not in candidates:
                candidates.append([phrase, phrase])
        except:
            self.logger.error(f"Cannot convert: {hiragana} {katakana}",
                              exc_info=True)

        if [katakana, katakana] not in candidates:
            candidates.append([katakana, katakana])

        for e in [[x, x] for x in self.system_dict.get_candidates(src, hiragana)]:
            if e not in candidates:
                candidates.append(e)

        if src[0].isupper():
            # 先頭が大文字の場合、それを先頭にもってくる。
            candidates.insert(0, [src, src])
        else:
            # そうじゃなければ、末尾にいれる。
            candidates.append([src, src])

        return candidates
