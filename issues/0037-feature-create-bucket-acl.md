# CreateBucket の acl パラメータ対応

## 概要

CreateBucket に `acl` パラメータを追加する。

## 対象 API

- CreateBucket

## 説明

バケット作成時に ACL (private, public-read 等) を指定できるようにする。

## 現在の実装状況

- `CreateBucketFluentBuilder` (`src/api/create_bucket.rs`) に `acl` フィールドは未実装
- 他の API (PutObject, CopyObject, CreateMultipartUpload) では `acl` フィールドと `x-amz-acl` ヘッダー送信は実装済み

## 優先度

低
