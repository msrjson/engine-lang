<?php

use Illuminate\Support\Facades\Route;
use Illuminate\Support\Facades\DB;
use Illuminate\Http\Request;

Route::get('/', function () {
    return view('welcome');
});

Route::get('/api/v1/categories', function () {
    $categories = DB::table('categories')
        ->select('id', 'name', 'slug', 'post_count')
        ->get();
    return response()->json($categories);
});

Route::get('/api/v1/posts', function () {
    $posts = DB::table('posts')
        ->select('id', 'title', 'slug', 'category', 'author', 'content', 'views', 'epoch')
        ->get();
    return response()->json($posts);
});

Route::post('/api/v1/auth/login', function (Request $request) {
    return response()->json([
        'status' => 'success',
        'message' => 'Autenticado com sucesso',
        'user' => 'autor@engine-lang.org',
        'role' => 'Author'
    ]);
});

Route::post('/api/v1/posts', function (Request $request) {
    $title = $request->input('title', 'Novo Artigo Laravel');
    $slug = $request->input('slug', 'novo-artigo-' . uniqid());
    $category = $request->input('category', 'Sistemas Distribuídos');
    $content = $request->input('content', 'Conteúdo persistido via SQLite no Laravel.');

    DB::transaction(function () use ($title, $slug, $category, $content) {
        $count = DB::table('posts')->count();
        DB::table('posts')->insert([
            'id' => 'post_' . ($count + 1),
            'title' => $title,
            'slug' => $slug,
            'category' => $category,
            'author' => 'Alan Kay',
            'content' => $content,
            'views' => 1,
            'epoch' => $count + 1,
            'created_at' => now(),
            'updated_at' => now(),
        ]);

        DB::table('categories')
            ->where('name', $category)
            ->increment('post_count');
    });

    return redirect('/', 303);
});
