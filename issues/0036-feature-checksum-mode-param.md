# GetObject / HeadObject の checksum_mode パラメータ対応

## 概要

GetObject と HeadObject に `checksum_mode` パラメータを追加する。

## 対象 API

- GetObject
- HeadObject

## 説明

`checksum_mode=ENABLED` を指定すると、レスポンスにオブジェクトのチェックサム値が含まれる。データ整合性の検証に使用する。

## 現在の実装状況

- `GetObjectFluentBuilder` (`src/api/get_object.rs`) に `checksum_mode` フィールドは未実装
- `HeadObjectFluentBuilder` (`src/api/head_object.rs`) に `checksum_mode` フィールドは未実装
- レスポンスの `GetObjectOutput` / `HeadObjectOutput` にもチェックサム値フィールドは未実装

## 優先度

低
