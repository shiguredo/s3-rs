# CreateBucket の acl パラメータ対応

Created: 2026-04-04
Completed: 2026-04-05
Model: Opus 4.6

## 概要

CreateBucket に `acl` パラメータを追加する。

## 対象 API

- CreateBucket

## 説明

バケット作成時に ACL (private, public-read 等) を指定できるようにする。

## 優先度

低

## 解決方法

`CreateBucketFluentBuilder` に `acl: Option<String>` フィールドとセッターメソッドを追加し、`x-amz-acl` ヘッダーとして出力するようにした。
