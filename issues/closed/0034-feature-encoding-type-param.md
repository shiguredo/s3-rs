# ListObjectsV2 / ListMultipartUploads の encoding_type パラメータ対応

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 概要

ListObjectsV2 と ListMultipartUploads に `encoding_type` パラメータを追加する。

## 対象 API

- ListObjectsV2
- ListMultipartUploads

## 説明

`encoding_type=url` を指定すると、レスポンス内のキー名が URL エンコードされて返される。キー名に XML で表現できない文字が含まれる場合に必要。

## 優先度

低

## 解決方法

`ListObjectsV2FluentBuilder` と `ListMultipartUploadsFluentBuilder` の両方に `encoding_type` フィールドとセッターメソッドを追加し、クエリパラメータとして出力するようにした。
