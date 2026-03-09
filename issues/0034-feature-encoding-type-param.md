# ListObjectsV2 / ListMultipartUploads の encoding_type パラメータ対応

## 概要

ListObjectsV2 と ListMultipartUploads に `encoding_type` パラメータを追加する。

## 対象 API

- ListObjectsV2
- ListMultipartUploads

## 説明

`encoding_type=url` を指定すると、レスポンス内のキー名が URL エンコードされて返される。キー名に XML で表現できない文字が含まれる場合に必要。

## 現在の実装状況

- `ListObjectsV2FluentBuilder` (`src/api/list_objects_v2.rs`) に `encoding_type` フィールドは未実装
- `ListMultipartUploadsFluentBuilder` (`src/api/list_multipart_uploads.rs`) に `encoding_type` フィールドは未実装

## 優先度

低
