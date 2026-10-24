<?php

namespace Database\Seeders;

use Illuminate\Database\Seeder;
use Illuminate\Support\Facades\DB;

class DatabaseSeeder extends Seeder
{
    public function run(): void
    {
        $categories = [
            ['id' => 'cat_spec', 'name' => 'Especificação & EBNF', 'slug' => 'especificacao-ebnf', 'post_count' => 1],
            ['id' => 'cat_op', 'name' => 'Operadores & SIMD', 'slug' => 'operadores-simd', 'post_count' => 1],
            ['id' => 'cat_sls', 'name' => 'Arquitetura Engine DB & SLS', 'slug' => 'engine-db-sls', 'post_count' => 1],
            ['id' => 'cat_evol', 'name' => 'Evolução de Esquema LSE-CRC', 'slug' => 'evolucao-esquema', 'post_count' => 1],
            ['id' => 'cat_mem', 'name' => 'Modelo de Memória & Concorrência', 'slug' => 'modelo-memoria', 'post_count' => 1],
            ['id' => 'cat_mesh', 'name' => 'Malha P2P & Wire Protocol', 'slug' => 'malha-p2p', 'post_count' => 1],
            ['id' => 'cat_mod', 'name' => 'Módulos & Pacotes', 'slug' => 'modulos-pacotes', 'post_count' => 1],
            ['id' => 'cat_smt', 'name' => 'Verificação Formal SMT & Z3', 'slug' => 'verificacao-smt', 'post_count' => 1],
            ['id' => 'cat_ffi', 'name' => 'FFI C & Ponte PostgreSQL', 'slug' => 'ffi-ponte-postgres', 'post_count' => 1],
            ['id' => 'cat_cases', 'name' => 'Casos de Uso & Engenharia', 'slug' => 'casos-de-uso', 'post_count' => 1],
            ['id' => 'cat_road', 'name' => 'Roadmap Estratégico (Fases 1 a 5)', 'slug' => 'roadmap-estrategico', 'post_count' => 1],
        ];

        foreach ($categories as $cat) {
            DB::table('categories')->insert(array_merge($cat, [
                'created_at' => now(),
                'updated_at' => now(),
            ]));
        }

        $posts = [
            [
                'id' => 'doc_1',
                'title' => '01. Especificação da Linguagem: Gramática Formal EBNF, Diretivas e Pattern Matching',
                'slug' => '01-especificacao-formal-ebnf',
                'category' => 'Especificação & EBNF',
                'author' => 'Engine Lang Core Working Group',
                'content' => 'O Engine Lang adota uma gramática estritamente determinística LL(1) sem backtracking, eliminando ambiguidade de parsing.',
                'views' => 124,
                'epoch' => 1
            ],
            [
                'id' => 'doc_2',
                'title' => '02. Matriz Fechada de 14 Níveis de Operadores: SIMD, Causalidade e Transações',
                'slug' => '02-matriz-operadores-simd-causal',
                'category' => 'Operadores & SIMD',
                'author' => 'Compiler Architecture Team',
                'content' => 'O Engine Lang define uma matriz fechada de operadores com 14 níveis estritos de precedência (ENG-SPEC-03.1).',
                'views' => 98,
                'epoch' => 2
            ],
            [
                'id' => 'doc_3',
                'title' => '03. Layout Binário Físico do Storage .db (ENG-SLS-01.1): O Fim do Banco de Dados Externo',
                'slug' => '03-layout-binario-db-sls',
                'category' => 'Arquitetura Engine DB & SLS',
                'author' => 'Systems & Storage Division',
                'content' => 'A Single-Level Store (SLS) mapeia o arquivo .db (engine.db) diretamente no espaço de endereçamento do processo via DAX.',
                'views' => 142,
                'epoch' => 3
            ],
            [
                'id' => 'doc_4',
                'title' => '04. Evolução de Esquema Zero-Downtime: O Protocolo LSE-CRC sem Migração SQL',
                'slug' => '04-evolucao-esquema-zero-downtime',
                'category' => 'Evolução de Esquema LSE-CRC',
                'author' => 'Database & Distributed Systems Group',
                'content' => 'Como o Engine Lang realiza upgrades de esquema sem perda de dados e sem travar a aplicação.',
                'views' => 185,
                'epoch' => 4
            ],
            [
                'id' => 'doc_5',
                'title' => '05. Modelo de Memória Causal: Barreira de Hardware, CLWB e SFENCE',
                'slug' => '05-modelo-memoria-causal-hardware',
                'category' => 'Modelo de Memória & Concorrência',
                'author' => 'Hardware Interop Team',
                'content' => 'O modelo de concorrência do Engine Lang estabelece consistência causal sequencial sem locks pessimistas de banco de dados.',
                'views' => 171,
                'epoch' => 5
            ],
            [
                'id' => 'doc_6',
                'title' => '06. Malha Causal P2P: Replicação sem Coordenação Central e sem 2PC',
                'slug' => '06-malha-causal-p2p-crtd',
                'category' => 'Malha P2P & Wire Protocol',
                'author' => 'Network Protocols Group',
                'content' => 'O Engine Lang implementa replicação causal assíncrona entre nós com delta-CRDTs e relógios vetoriais.',
                'views' => 110,
                'epoch' => 6
            ],
            [
                'id' => 'doc_7',
                'title' => '07. Módulos, Pacotes e os 4 Níveis de Visibilidade no Engine Lang',
                'slug' => '07-modulos-pacotes-visibilidade',
                'category' => 'Módulos & Pacotes',
                'author' => 'Language Design & Tooling',
                'content' => 'A arquitetura de código do Engine Lang promove encapsulamento estrito e modularidade composicional.',
                'views' => 74,
                'epoch' => 7
            ],
            [
                'id' => 'doc_8',
                'title' => '08. Tipos Refinados e Verificação Formal SMT: Eliminando Bugs com Provas Matemáticas',
                'slug' => '08-tipos-refinados-verificacao-smt',
                'category' => 'Verificação Formal SMT & Z3',
                'author' => 'Formal Methods Laboratory',
                'content' => 'O Engine Lang substitui testes unitários frágeis por provas formais estáticas em tempo de compilação.',
                'views' => 163,
                'epoch' => 8
            ],
            [
                'id' => 'doc_9',
                'title' => '09. Interoperabilidade FFI C e Virtual Bridge PostgreSQL: Conectando Ferramentas Legadas',
                'slug' => '09-ffi-c-ponte-virtual-postgresql',
                'category' => 'FFI C & Ponte PostgreSQL',
                'author' => 'Ecosystem Integration Division',
                'content' => 'Como modernizar sistemas sem reescrever ferramentas de BI e análise corporativas.',
                'views' => 129,
                'epoch' => 9
            ],
            [
                'id' => 'doc_10',
                'title' => '10. Casos de Uso Arquiteturais: FinTech de Alta Frequência, Frotas IoT e IA Vetorial',
                'slug' => '10-casos-de-uso-arquiteturais-tco',
                'category' => 'Casos de Uso & Engenharia',
                'author' => 'Solutions Architecture Team',
                'content' => 'O Engine Lang elimina o chamado Imposto da Fragmentação na Nuvem, unificando computação e estado em um único runtime.',
                'views' => 215,
                'epoch' => 10
            ],
            [
                'id' => 'doc_11',
                'title' => '11. Roadmap Estratégico do Engine Lang: Do Nó Genesis v0.3.0 à Certificação ISO',
                'slug' => '11-roadmap-estrategico-fases',
                'category' => 'Roadmap Estratégico (Fases 1 a 5)',
                'author' => 'Steering Committee',
                'content' => 'O plano de engenharia do ecossistema Engine Lang está estruturado em 5 fases de maturação industrial.',
                'views' => 95,
                'epoch' => 11
            ],
        ];

        foreach ($posts as $post) {
            DB::table('posts')->insert(array_merge($post, [
                'created_at' => now(),
                'updated_at' => now(),
            ]));
        }
    }
}
