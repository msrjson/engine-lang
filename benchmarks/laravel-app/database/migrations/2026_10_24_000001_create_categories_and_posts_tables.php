<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('categories', function (Blueprint $table) {
            $table->string('id')->primary();
            $table->string('name');
            $table->string('slug')->unique();
            $table->unsignedBigInteger('post_count')->default(0);
            $table->timestamps();
        });

        Schema::create('posts', function (Blueprint $table) {
            $table->string('id')->primary();
            $table->string('title');
            $table->string('slug')->unique();
            $table->string('category');
            $table->string('author');
            $table->text('content');
            $table->unsignedBigInteger('views')->default(1);
            $table->unsignedBigInteger('epoch')->default(1);
            $table->timestamps();
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('posts');
        Schema::dropIfExists('categories');
    }
};

