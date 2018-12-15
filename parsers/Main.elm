module Main exposing (..)

import Browser
import Html exposing (Html, button, div, text, input, a, header, p, hr)
import Html.Attributes exposing (placeholder, value, class, href)
import Html.Events exposing (onClick, onInput)
import Json.Decode as Decode
import Http


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.none


init : Int -> ( Model, Cmd Msg )
init _ =
    ( { query = "", results = Nothing }, Cmd.none )


type Msg
    = Search
    | UpdateQuery String
    | NewResult (Result Http.Error (List SearchResult))


type alias TypeSignature =
    String


type alias Model =
    { query : TypeSignature
    , results : Maybe (List SearchResult)
    }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Search ->
            ( model
            , Http.send NewResult
                (Http.get
                    ("http://localhost:8000/search/" ++ model.query)
                    searchResultDecoder
                )
            )

        UpdateQuery query ->
            ( { model | query = query }, Cmd.none )

        NewResult result ->
            case result of
                Ok data ->
                    ( { model | results = Just data }, Cmd.none )

                Err _ ->
                    ( model, Cmd.none )


functionView : SearchResult -> Html Msg
functionView res =
    div [ class "card" ]
        [ header [ class "card-header" ]
            [ p [ class "card-header-title" ]
                [ text (res.fn.name ++ " : " ++ (res.fn.args |> String.join " -> ")) ]
            ]
        , div [ class "card-content" ]
            [ div [ class "content" ]
                [ text res.fn.desc
                , text ". "
                , a [ href res.repo.url ]
                    [ text res.repo.name ]
                ]
            ]
        ]


view : Model -> Html Msg
view model =
    let
        results =
            case model.results of
                Just r ->
                    r |> List.map functionView

                Nothing ->
                    [ div [] [] ]
    in
        div [ class "columns" ]
            [ div [ class "column is-10 is-offset-1" ]
                [ text "Elm TypeSignature Search Engine"
                , hr [] []
                , input
                    [ placeholder "Text to reverse"
                    , value model.query
                    , class "input"
                    , onInput
                        UpdateQuery
                    ]
                    []
                , button [ onClick Search, class "button" ] [ text "Search" ]
                , hr [] []
                , div [] results
                ]
            ]


type alias SearchResultRepo =
    { name : String, url : String }


type alias SearchResultFn =
    { name : String, desc : String, args : List String }


type alias SearchResult =
    { repo : SearchResultRepo, fn : SearchResultFn }


searchResultDecoder : Decode.Decoder (List SearchResult)
searchResultDecoder =
    Decode.at [ "data" ]
        (Decode.map2 SearchResult
            (Decode.at [ "repo" ] repoDecoder)
            (Decode.at [ "res" ] resDecoder)
            |> Decode.list
        )


repoDecoder : Decode.Decoder SearchResultRepo
repoDecoder =
    Decode.map2 SearchResultRepo
        (Decode.at [ "name" ] Decode.string)
        (Decode.at [ "url" ] Decode.string)


resDecoder : Decode.Decoder SearchResultFn
resDecoder =
    Decode.map3 SearchResultFn
        (Decode.at [ "name" ] Decode.string)
        (Decode.at [ "desc" ] Decode.string)
        (Decode.at [ "args" ] (Decode.list Decode.string))


main =
    Browser.element
        { view = view
        , init = init
        , update = update
        , subscriptions = subscriptions
        }
