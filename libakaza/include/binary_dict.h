#pragma once

#include <cstring>
#include <iostream>
#include <string>
#include <vector>
#include <sstream>
#include <tuple>

#include <marisa.h>


namespace akaza {
    class BinaryDict {
    private:
        marisa::Trie dict_trie;

        std::vector<std::string> split(const std::string &s) {
            std::vector<std::string> elems;
            std::stringstream ss(s);
            std::string item;
            while (getline(ss, item, '/')) {
                if (!item.empty()) {
                    elems.push_back(item);
                }
            }
            return elems;
        }

    public:
        BinaryDict() {}

        void load(const std::string& dict_path);

        void save(std::string dict_path) {
            dict_trie.save(dict_path.c_str());
            std::cout << "[Save] " << dict_path << ": " << dict_trie.num_keys() << std::endl;
        }

        void build_by_keyset(marisa::Keyset &keyset) {
            dict_trie.build(keyset);
        }

        // vector of "とくひろ" => "徳宏/徳大/徳寛/督弘"
        void build(std::vector<std::tuple<std::string, std::string>> data) {
            marisa::Keyset keyset;
            for (auto &d: data) {
                std::string yomi = std::get<0>(d);
                std::string kanjis = std::get<1>(d);
                keyset.push_back((yomi + "\xff" + kanjis).c_str());
            }
            this->build_by_keyset(keyset);
        }

        std::vector<std::string> find_kanjis(std::string word) {
            std::string query(word);
            query += "\xff"; // add marker

            marisa::Agent agent;
            agent.set_query(query.c_str(), query.size());

            while (dict_trie.predictive_search(agent)) {
                std::string kanjis = std::string(agent.key().ptr() + query.size(), agent.key().length() - query.size());
                return split(kanjis);
            }
            return std::vector<std::string>();
        }

    };
} // namespace akaza
